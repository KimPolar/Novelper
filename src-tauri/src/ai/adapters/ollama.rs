use crate::ai::adapters::common::{build_http_client, is_retryable_status, merge_json_object};
use crate::ai::{
    AIError, AIEvent, AIModelInfo, AIProvider, AIProviderConfig, AIRequest, AIResponse,
    ModelCapabilities, ProtocolKind,
};
use futures_util::StreamExt;
use serde_json::{json, Value};
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

pub struct OllamaAdapter {
    config: AIProviderConfig,
    client: reqwest::Client,
}

impl OllamaAdapter {
    pub fn new(config: AIProviderConfig) -> Result<Self, AIError> {
        let client = build_http_client(&config)?;
        Ok(Self { config, client })
    }

    fn base_url(&self) -> String {
        self.config
            .base_url
            .clone()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "http://localhost:11434".to_string())
            .trim_end_matches('/')
            .to_string()
    }

    pub fn build_chat_body(&self, req: &AIRequest, stream: bool) -> Value {
        let mut messages = Vec::new();
        if let Some(sys) = &req.system {
            if !sys.is_empty() {
                messages.push(json!({ "role": "system", "content": sys }));
            }
        }
        for m in &req.messages {
            messages.push(json!({ "role": m.role, "content": m.content }));
        }

        let mut options = json!({});
        if let Some(temp) = req.temperature {
            options["temperature"] = json!(temp);
        }
        if let Some(max) = req.max_tokens {
            options["num_predict"] = json!(max);
        }
        if let Some(ns_opts) = req.provider_options.pointer("/ollama/options") {
            merge_json_object(&mut options, ns_opts);
        }

        let mut body = json!({
            "model": req.model_id,
            "messages": messages,
            "stream": stream,
        });
        if options.as_object().map(|m| !m.is_empty()).unwrap_or(false) {
            body["options"] = options;
        }
        if let Some(ns) = req.provider_options.get("ollama") {
            let mut filtered = ns.clone();
            if let Some(obj) = filtered.as_object_mut() {
                obj.remove("options");
            }
            merge_json_object(&mut body, &filtered);
        }
        body
    }
}

pub fn parse_ollama_ndjson_line(line: &str) -> Option<AIEvent> {
    let v: Value = serde_json::from_str(line).ok()?;
    if let Some(content) = v.pointer("/message/content").and_then(|c| c.as_str()) {
        if !content.is_empty() {
            return Some(AIEvent::TextDelta {
                delta: content.to_string(),
            });
        }
    }
    if v.get("done").and_then(|d| d.as_bool()) == Some(true) {
        return Some(AIEvent::Done {
            finish_reason: v
                .get("done_reason")
                .and_then(|r| r.as_str())
                .unwrap_or("stop")
                .to_string(),
            provider_meta: v,
        });
    }
    None
}

#[async_trait::async_trait]
impl AIProvider for OllamaAdapter {
    fn id(&self) -> &str {
        &self.config.id
    }

    fn protocol(&self) -> ProtocolKind {
        ProtocolKind::OllamaNative
    }

    async fn list_models(&self) -> Result<Vec<AIModelInfo>, AIError> {
        let url = format!("{}/api/tags", self.base_url());
        let resp = self.client.get(&url).send().await?;
        let status = resp.status().as_u16();
        if !resp.status().is_success() {
            let msg = resp.text().await.unwrap_or_default();
            return Err(AIError::Provider {
                status,
                message: msg,
                retryable: is_retryable_status(status),
            });
        }
        let body: Value = resp.json().await?;
        let models = body
            .get("models")
            .and_then(|m| m.as_array())
            .cloned()
            .unwrap_or_default();
        Ok(models
            .into_iter()
            .filter_map(|item| {
                let name = item.get("name")?.as_str()?.to_string();
                Some(AIModelInfo {
                    id: format!("{}:{}", self.config.id, name),
                    provider_id: self.config.id.clone(),
                    model_id: name.clone(),
                    display_name: name.clone(),
                    capabilities: self.get_capabilities(&name),
                    default_options: None,
                    is_custom: false,
                })
            })
            .collect())
    }

    fn get_capabilities(&self, _model_id: &str) -> ModelCapabilities {
        ModelCapabilities {
            streaming: true,
            vision: false,
            tool_calling: true,
            structured_output: true,
            reasoning: false,
            prompt_caching: false,
            context_window: Some(32_768),
            max_output_tokens: Some(8_192),
        }
    }

    async fn generate(&self, req: AIRequest) -> Result<AIResponse, AIError> {
        let url = format!("{}/api/chat", self.base_url());
        let body = self.build_chat_body(&req, false);
        let resp = self.client.post(&url).json(&body).send().await?;
        let status = resp.status().as_u16();
        if !resp.status().is_success() {
            let msg = resp.text().await.unwrap_or_default();
            return Err(AIError::Provider {
                status,
                message: msg,
                retryable: is_retryable_status(status),
            });
        }
        let v: Value = resp.json().await?;
        let text = v
            .pointer("/message/content")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();
        Ok(AIResponse {
            text,
            reasoning: None,
            input_tokens: v
                .get("prompt_eval_count")
                .and_then(|x| x.as_u64())
                .map(|x| x as u32),
            output_tokens: v
                .get("eval_count")
                .and_then(|x| x.as_u64())
                .map(|x| x as u32),
            finish_reason: "stop".to_string(),
            provider_meta: v,
        })
    }

    async fn stream(
        &self,
        req: AIRequest,
        tx: Channel<AIEvent>,
        cancel: CancellationToken,
    ) -> Result<(), AIError> {
        let url = format!("{}/api/chat", self.base_url());
        let body = self.build_chat_body(&req, true);
        let resp = self.client.post(&url).json(&body).send().await?;
        let status = resp.status().as_u16();
        if !resp.status().is_success() {
            let msg = resp.text().await.unwrap_or_default();
            return Err(AIError::Provider {
                status,
                message: msg,
                retryable: is_retryable_status(status),
            });
        }

        let mut byte_stream = resp.bytes_stream();
        let mut buffer = String::new();
        loop {
            tokio::select! {
                _ = cancel.cancelled() => {
                    let _ = tx.send(AIEvent::Error {
                        code: "cancelled".to_string(),
                        message: "Request cancelled by user".to_string(),
                        retryable: false,
                    });
                    return Err(AIError::Cancelled);
                }
                chunk = byte_stream.next() => {
                    match chunk {
                        Some(Ok(bytes)) => {
                            buffer.push_str(&String::from_utf8_lossy(&bytes));
                            while let Some(pos) = buffer.find('\n') {
                                let line = buffer[..pos].trim().to_string();
                                buffer.drain(..=pos);
                                if line.is_empty() { continue; }
                                if let Some(ev) = parse_ollama_ndjson_line(&line) {
                                    let is_done = matches!(ev, AIEvent::Done { .. });
                                    let _ = tx.send(ev);
                                    if is_done {
                                        return Ok(());
                                    }
                                }
                            }
                        }
                        Some(Err(e)) => return Err(AIError::Http(e)),
                        None => break,
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::AIMessage;

    #[test]
    fn test_ollama_body_and_ndjson() {
        let cfg = AIProviderConfig {
            id: "ollama".into(),
            name: "Ollama".into(),
            protocol: ProtocolKind::OllamaNative,
            base_url: Some("http://localhost:11434".into()),
            secret_ref: None,
            extra_headers: None,
            timeout_ms: 30_000,
            max_retries: 1,
            is_local: true,
            enabled: true,
        };
        let adapter = OllamaAdapter::new(cfg).unwrap();
        let req = AIRequest {
            model_id: "qwen3:8b".into(),
            system: Some("소설가".into()),
            messages: vec![AIMessage {
                role: "user".into(),
                content: "다음 문장".into(),
            }],
            temperature: Some(0.8),
            max_tokens: Some(512),
            tools: vec![],
            provider_options: json!({
                "ollama": { "keep_alive": "10m", "options": { "num_ctx": 16384 } }
            }),
        };
        let body = adapter.build_chat_body(&req, true);
        assert_eq!(body["keep_alive"], "10m");
        assert_eq!(body["options"]["num_ctx"], 16384);

        let ev = parse_ollama_ndjson_line(r#"{"message":{"content":"어둠"},"done":false}"#);
        assert!(matches!(ev, Some(AIEvent::TextDelta { delta }) if delta == "어둠"));
    }
}
