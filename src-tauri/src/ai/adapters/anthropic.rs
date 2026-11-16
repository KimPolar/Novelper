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

pub struct AnthropicAdapter {
    config: AIProviderConfig,
    api_key: Option<String>,
    client: reqwest::Client,
}

impl AnthropicAdapter {
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
            .unwrap_or_else(|| "https://api.anthropic.com/v1".to_string())
            .trim_end_matches('/')
            .to_string()
    }

    pub fn build_body(&self, req: &AIRequest, stream: bool) -> Value {
        let messages: Vec<Value> = req
            .messages
            .iter()
            .filter(|m| m.role != "system")
            .map(|m| json!({ "role": m.role, "content": m.content }))
            .collect();

        let mut body = json!({
            "model": req.model_id,
            "messages": messages,
            "max_tokens": req.max_tokens.unwrap_or(4096),
            "stream": stream,
        });

        if let Some(sys) = &req.system {
            if !sys.is_empty() {
                if let Some(cache_ctrl) = req.provider_options.pointer("/anthropic/cache_control") {
                    body["system"] = json!([{
                        "type": "text",
                        "text": sys,
                        "cache_control": cache_ctrl
                    }]);
                } else {
                    body["system"] = json!(sys);
                }
            }
        }
        if let Some(temp) = req.temperature {
            body["temperature"] = json!(temp);
        }
        if !req.tools.is_empty() {
            body["tools"] = json!(req
                .tools
                .iter()
                .map(|t| json!({
                    "name": t.name,
                    "description": t.description,
                    "input_schema": t.parameters_schema
                }))
                .collect::<Vec<_>>());
        }
        if let Some(ns) = req.provider_options.get("anthropic") {
            let mut filtered = ns.clone();
            if let Some(obj) = filtered.as_object_mut() {
                obj.remove("cache_control");
            }
            merge_json_object(&mut body, &filtered);
        }
        body
    }

    fn apply_headers(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        let b = builder.header("anthropic-version", "2023-06-01");
        if let Some(key) = &self.api_key {
            b.header("x-api-key", key)
        } else {
            b
        }
    }
}

pub fn parse_anthropic_sse_payload(payload: &str) -> Option<AIEvent> {
    let v: Value = serde_json::from_str(payload).ok()?;
    let ev_type = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
    match ev_type {
        "content_block_delta" => {
            let delta = v.get("delta")?;
            let d_type = delta.get("type").and_then(|t| t.as_str()).unwrap_or("");
            if d_type == "text_delta" {
                let text = delta.get("text").and_then(|t| t.as_str()).unwrap_or("");
                if !text.is_empty() {
                    return Some(AIEvent::TextDelta {
                        delta: text.to_string(),
                    });
                }
            } else if d_type == "thinking_delta" {
                let thinking = delta.get("thinking").and_then(|t| t.as_str()).unwrap_or("");
                if !thinking.is_empty() {
                    return Some(AIEvent::ReasoningDelta {
                        delta: thinking.to_string(),
                    });
                }
            }
        }
        "message_delta" => {
            if let Some(usage) = v.get("usage") {
                return Some(AIEvent::Usage {
                    input_tokens: usage
                        .get("input_tokens")
                        .and_then(|x| x.as_u64())
                        .unwrap_or(0) as u32,
                    output_tokens: usage
                        .get("output_tokens")
                        .and_then(|x| x.as_u64())
                        .unwrap_or(0) as u32,
                });
            }
        }
        "message_stop" => {
            return Some(AIEvent::Done {
                finish_reason: "stop".to_string(),
                provider_meta: Value::Null,
            });
        }
        _ => {}
    }
    None
}

#[async_trait::async_trait]
impl AIProvider for AnthropicAdapter {
    fn id(&self) -> &str {
        &self.config.id
    }

    fn protocol(&self) -> ProtocolKind {
        ProtocolKind::AnthropicMessages
    }

    async fn list_models(&self) -> Result<Vec<AIModelInfo>, AIError> {
        let url = format!("{}/models", self.base_url());
        let resp = self.apply_headers(self.client.get(&url)).send().await?;
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
                let name = item
                    .get("display_name")
                    .and_then(|n| n.as_str())
                    .unwrap_or(&id)
                    .to_string();
                Some(AIModelInfo {
                    id: format!("{}:{}", self.config.id, id),
                    provider_id: self.config.id.clone(),
                    model_id: id.clone(),
                    display_name: name,
                    capabilities: self.get_capabilities(&id),
                    default_options: None,
                    is_custom: false,
                })
            })
            .collect())
    }

    fn get_capabilities(&self, _model_id: &str) -> ModelCapabilities {
        ModelCapabilities {
            streaming: true,
            vision: true,
            tool_calling: true,
            structured_output: true,
            reasoning: true,
            prompt_caching: true,
            context_window: Some(200_000),
            max_output_tokens: Some(16_384),
        }
    }

    async fn generate(&self, req: AIRequest) -> Result<AIResponse, AIError> {
        let url = format!("{}/messages", self.base_url());
        let body = self.build_body(&req, false);
        let resp = self
            .apply_headers(self.client.post(&url))
            .json(&body)
            .send()
            .await?;
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
            .get("content")
            .and_then(|c| c.as_array())
            .map(|arr| {
                arr.iter()
                    .filter(|b| b.get("type").and_then(|t| t.as_str()) == Some("text"))
                    .filter_map(|b| b.get("text").and_then(|t| t.as_str()))
                    .collect::<Vec<_>>()
                    .join("")
            })
            .unwrap_or_default();
        Ok(AIResponse {
            text,
            reasoning: None,
            input_tokens: v
                .pointer("/usage/input_tokens")
                .and_then(|x| x.as_u64())
                .map(|x| x as u32),
            output_tokens: v
                .pointer("/usage/output_tokens")
                .and_then(|x| x.as_u64())
                .map(|x| x as u32),
            finish_reason: v
                .get("stop_reason")
                .and_then(|s| s.as_str())
                .unwrap_or("end_turn")
                .to_string(),
            provider_meta: v,
        })
    }

    async fn stream(
        &self,
        req: AIRequest,
        tx: Channel<AIEvent>,
        cancel: CancellationToken,
    ) -> Result<(), AIError> {
        let url = format!("{}/messages", self.base_url());
        let body = self.build_body(&req, true);
        let resp = self
            .apply_headers(self.client.post(&url))
            .json(&body)
            .send()
            .await?;
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
            if let Some(ev) = parse_anthropic_sse_payload(payload) {
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
    fn test_anthropic_prompt_caching_and_sse() {
        let cfg = AIProviderConfig {
            id: "claude".into(),
            name: "Anthropic".into(),
            protocol: ProtocolKind::AnthropicMessages,
            base_url: None,
            secret_ref: None,
            extra_headers: None,
            timeout_ms: 30_000,
            max_retries: 2,
            is_local: false,
            enabled: true,
        };
        let adapter = AnthropicAdapter::new(cfg, Some("sk-ant".into())).unwrap();
        let req = AIRequest {
            model_id: "claude-sonnet-4-5".into(),
            system: Some("Worldbuilding Bible".into()),
            messages: vec![AIMessage {
                role: "user".into(),
                content: "Expand scene".into(),
            }],
            temperature: Some(0.7),
            max_tokens: Some(2048),
            tools: vec![],
            provider_options: json!({
                "anthropic": {
                    "cache_control": { "type": "ephemeral" }
                }
            }),
        };
        let body = adapter.build_body(&req, true);
        assert_eq!(body["system"][0]["cache_control"]["type"], "ephemeral");

        let ev = parse_anthropic_sse_payload(
            r#"{"type":"content_block_delta","delta":{"type":"text_delta","text":"별빛"}}"#,
        );
        assert!(matches!(ev, Some(AIEvent::TextDelta { delta }) if delta == "별빛"));
    }
}
