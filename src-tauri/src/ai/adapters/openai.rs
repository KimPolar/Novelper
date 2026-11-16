use crate::ai::adapters::common::{
    build_http_client, is_retryable_status, merge_json_object, stream_sse_lines,
};
use crate::ai::{
    AIError, AIEvent, AIModelInfo, AIProvider, AIProviderConfig, AIRequest, AIResponse,
    ModelCapabilities, ProtocolKind,
};
use serde_json::{json, Value};
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

pub struct OpenAIAdapter {
    config: AIProviderConfig,
    api_key: Option<String>,
    client: reqwest::Client,
}

impl OpenAIAdapter {
    pub fn new(config: AIProviderConfig, api_key: Option<String>) -> Result<Self, AIError> {
        let client = build_http_client(&config)?;
        Ok(Self {
            config,
            api_key,
            client,
        })
    }

    fn base_url(&self) -> String {
        self.config
            .base_url
            .clone()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string())
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

        let mut body = json!({
            "model": req.model_id,
            "messages": messages,
            "stream": stream,
        });
        if let Some(temp) = req.temperature {
            body["temperature"] = json!(temp);
        }
        if let Some(max) = req.max_tokens {
            body["max_tokens"] = json!(max);
        }
        if !req.tools.is_empty() {
            body["tools"] = json!(req
                .tools
                .iter()
                .map(|t| json!({
                    "type": "function",
                    "function": {
                        "name": t.name,
                        "description": t.description,
                        "parameters": t.parameters_schema
                    }
                }))
                .collect::<Vec<_>>());
        }
        if let Some(ns) = req.provider_options.get("openai_chat") {
            merge_json_object(&mut body, ns);
        }
        body
    }

    pub fn build_responses_body(&self, req: &AIRequest, stream: bool) -> Value {
        let input: Vec<Value> = req
            .messages
            .iter()
            .map(|m| json!({ "role": m.role, "content": m.content }))
            .collect();

        let mut body = json!({
            "model": req.model_id,
            "input": input,
            "stream": stream,
        });
        if let Some(sys) = &req.system {
            if !sys.is_empty() {
                body["instructions"] = json!(sys);
            }
        }
        if let Some(temp) = req.temperature {
            body["temperature"] = json!(temp);
        }
        if let Some(max) = req.max_tokens {
            body["max_output_tokens"] = json!(max);
        }
        if !req.tools.is_empty() {
            body["tools"] = json!(req
                .tools
                .iter()
                .map(|t| json!({
                    "type": "function",
                    "name": t.name,
                    "description": t.description,
                    "parameters": t.parameters_schema
                }))
                .collect::<Vec<_>>());
        }
        if let Some(ns) = req.provider_options.get("openai_responses") {
            merge_json_object(&mut body, ns);
        }
        body
    }

    fn auth_header(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(key) = &self.api_key {
            if !key.is_empty() {
                return builder.bearer_auth(key);
            }
        }
        builder
    }
}

pub fn parse_openai_sse_payload(payload: &str, is_responses: bool) -> Option<AIEvent> {
    if payload == "[DONE]" {
        return Some(AIEvent::Done {
            finish_reason: "stop".to_string(),
            provider_meta: Value::Null,
        });
    }
    let v: Value = serde_json::from_str(payload).ok()?;

    if is_responses {
        let event_type = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
        match event_type {
            "response.output_text.delta" => {
                let delta = v.get("delta").and_then(|d| d.as_str()).unwrap_or("");
                if !delta.is_empty() {
                    return Some(AIEvent::TextDelta {
                        delta: delta.to_string(),
                    });
                }
            }
            "response.reasoning_summary_text.delta" => {
                let delta = v.get("delta").and_then(|d| d.as_str()).unwrap_or("");
                if !delta.is_empty() {
                    return Some(AIEvent::ReasoningDelta {
                        delta: delta.to_string(),
                    });
                }
            }
            "response.completed" => {
                return Some(AIEvent::Done {
                    finish_reason: "completed".to_string(),
                    provider_meta: v.get("response").cloned().unwrap_or(Value::Null),
                });
            }
            _ => {}
        }
        return None;
    }

    if let Some(choices) = v.get("choices").and_then(|c| c.as_array()) {
        if let Some(first) = choices.first() {
            if let Some(delta) = first.get("delta").and_then(|d| d.get("content")).and_then(|c| c.as_str()) {
                if !delta.is_empty() {
                    return Some(AIEvent::TextDelta {
                        delta: delta.to_string(),
                    });
                }
            }
            if let Some(reason) = first.get("finish_reason").and_then(|r| r.as_str()) {
                return Some(AIEvent::Done {
                    finish_reason: reason.to_string(),
                    provider_meta: Value::Null,
                });
            }
        }
    }
    None
}

#[async_trait::async_trait]
impl AIProvider for OpenAIAdapter {
    fn id(&self) -> &str {
        &self.config.id
    }

    fn protocol(&self) -> ProtocolKind {
        self.config.protocol.clone()
    }

    async fn list_models(&self) -> Result<Vec<AIModelInfo>, AIError> {
        let url = format!("{}/models", self.base_url());
        let resp = self.auth_header(self.client.get(&url)).send().await?;
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
        let data = body
            .get("data")
            .and_then(|d| d.as_array())
            .cloned()
            .unwrap_or_default();
        Ok(data
            .into_iter()
            .filter_map(|item| {
                let id = item.get("id")?.as_str()?.to_string();
                Some(AIModelInfo {
                    id: format!("{}:{}", self.config.id, id),
                    provider_id: self.config.id.clone(),
                    model_id: id.clone(),
                    display_name: id.clone(),
                    capabilities: self.get_capabilities(&id),
                    default_options: None,
                    is_custom: false,
                })
            })
            .collect())
    }

    fn get_capabilities(&self, model_id: &str) -> ModelCapabilities {
        let lower = model_id.to_lowercase();
        ModelCapabilities {
            streaming: true,
            vision: lower.contains("gpt-4") || lower.contains("gpt-5") || lower.contains("o3"),
            tool_calling: true,
            structured_output: true,
            reasoning: lower.starts_with("o1") || lower.starts_with("o3") || lower.starts_with("o4") || lower.contains("gpt-5"),
            prompt_caching: true,
            context_window: Some(128_000),
            max_output_tokens: Some(16_384),
        }
    }

    async fn generate(&self, req: AIRequest) -> Result<AIResponse, AIError> {
        let is_responses = self.config.protocol == ProtocolKind::OpenaiResponses;
        let url = if is_responses {
            format!("{}/responses", self.base_url())
        } else {
            format!("{}/chat/completions", self.base_url())
        };
        let body = if is_responses {
            self.build_responses_body(&req, false)
        } else {
            self.build_chat_body(&req, false)
        };

        let resp = self.auth_header(self.client.post(&url)).json(&body).send().await?;
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
        if is_responses {
            let text = v
                .get("output_text")
                .and_then(|t| t.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| {
                    v.get("output")
                        .and_then(|o| o.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|item| item.get("content"))
                        .and_then(|c| c.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|c| c.get("text"))
                        .and_then(|t| t.as_str())
                        .unwrap_or("")
                        .to_string()
                });
            Ok(AIResponse {
                text,
                reasoning: None,
                input_tokens: v.pointer("/usage/input_tokens").and_then(|x| x.as_u64()).map(|x| x as u32),
                output_tokens: v.pointer("/usage/output_tokens").and_then(|x| x.as_u64()).map(|x| x as u32),
                finish_reason: v.get("status").and_then(|s| s.as_str()).unwrap_or("completed").to_string(),
                provider_meta: v,
            })
        } else {
            let text = v
                .pointer("/choices/0/message/content")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string();
            Ok(AIResponse {
                text,
                reasoning: None,
                input_tokens: v.pointer("/usage/prompt_tokens").and_then(|x| x.as_u64()).map(|x| x as u32),
                output_tokens: v.pointer("/usage/completion_tokens").and_then(|x| x.as_u64()).map(|x| x as u32),
                finish_reason: v
                    .pointer("/choices/0/finish_reason")
                    .and_then(|s| s.as_str())
                    .unwrap_or("stop")
                    .to_string(),
                provider_meta: v,
            })
        }
    }

    async fn stream(
        &self,
        req: AIRequest,
        tx: Channel<AIEvent>,
        cancel: CancellationToken,
    ) -> Result<(), AIError> {
        let is_responses = self.config.protocol == ProtocolKind::OpenaiResponses;
        let url = if is_responses {
            format!("{}/responses", self.base_url())
        } else {
            format!("{}/chat/completions", self.base_url())
        };
        let body = if is_responses {
            self.build_responses_body(&req, true)
        } else {
            self.build_chat_body(&req, true)
        };

        let resp = self.auth_header(self.client.post(&url)).json(&body).send().await?;
        let status = resp.status().as_u16();
        if !resp.status().is_success() {
            let msg = resp.text().await.unwrap_or_default();
            return Err(AIError::Provider {
                status,
                message: msg,
                retryable: is_retryable_status(status),
            });
        }

        stream_sse_lines(resp, &tx, &cancel, |payload, chan| {
            if let Some(ev) = parse_openai_sse_payload(payload, is_responses) {
                let is_done = matches!(ev, AIEvent::Done { .. });
                let _ = chan.send(ev);
                return Ok(is_done);
            }
            Ok(false)
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::AIMessage;

    #[test]
    fn test_openai_responses_preserves_namespace_options() {
        let cfg = AIProviderConfig {
            id: "oai".into(),
            name: "OpenAI".into(),
            protocol: ProtocolKind::OpenaiResponses,
            base_url: None,
            secret_ref: None,
            extra_headers: None,
            timeout_ms: 30_000,
            max_retries: 2,
            is_local: false,
            enabled: true,
        };
        let adapter = OpenAIAdapter::new(cfg, Some("sk-test".into())).unwrap();
        let req = AIRequest {
            model_id: "gpt-5".into(),
            system: Some("You are a novel editor".into()),
            messages: vec![AIMessage {
                role: "user".into(),
                content: "Check plot".into(),
            }],
            temperature: None,
            max_tokens: Some(1000),
            tools: vec![],
            provider_options: json!({
                "openai_responses": {
                    "reasoning": { "effort": "high" },
                    "previous_response_id": "resp_123"
                }
            }),
        };
        let body = adapter.build_responses_body(&req, true);
        assert_eq!(body["reasoning"]["effort"], "high");
        assert_eq!(body["previous_response_id"], "resp_123");
        assert_eq!(body["instructions"], "You are a novel editor");

        let ev = parse_openai_sse_payload(
            r#"{"type":"response.output_text.delta","delta":"안녕"}"#,
            true,
        );
        assert!(matches!(ev, Some(AIEvent::TextDelta { delta }) if delta == "안녕"));
    }
}
