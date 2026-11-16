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

pub struct GeminiAdapter {
    config: AIProviderConfig,
    api_key: Option<String>,
    client: reqwest::Client,
}

impl GeminiAdapter {
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
            .unwrap_or_else(|| "https://generativelanguage.googleapis.com/v1beta".to_string())
            .trim_end_matches('/')
            .to_string()
    }

    pub fn build_interactions_body(&self, req: &AIRequest, stream: bool) -> Value {
        let input: Vec<Value> = req
            .messages
            .iter()
            .map(|m| {
                json!({
                    "role": if m.role == "assistant" { "model" } else { "user" },
                    "content": [{ "type": "text", "text": m.content }]
                })
            })
            .collect();

        let mut body = json!({
            "model": req.model_id,
            "input": input,
            "stream": stream,
        });
        if let Some(sys) = &req.system {
            if !sys.is_empty() {
                body["system_instruction"] = json!(sys);
            }
        }
        let mut gen_cfg = json!({});
        if let Some(temp) = req.temperature {
            gen_cfg["temperature"] = json!(temp);
        }
        if let Some(max) = req.max_tokens {
            gen_cfg["max_output_tokens"] = json!(max);
        }
        if gen_cfg.as_object().map(|m| !m.is_empty()).unwrap_or(false) {
            body["generation_config"] = gen_cfg;
        }
        if let Some(ns) = req.provider_options.get("gemini_interactions") {
            merge_json_object(&mut body, ns);
        }
        body
    }

    pub fn build_generate_content_body(&self, req: &AIRequest) -> Value {
        let contents: Vec<Value> = req
            .messages
            .iter()
            .filter(|m| m.role != "system")
            .map(|m| {
                json!({
                    "role": if m.role == "assistant" { "model" } else { "user" },
                    "parts": [{ "text": m.content }]
                })
            })
            .collect();

        let mut body = json!({
            "contents": contents,
        });
        if let Some(sys) = &req.system {
            if !sys.is_empty() {
                body["systemInstruction"] = json!({
                    "parts": [{ "text": sys }]
                });
            }
        }
        let mut gen_cfg = json!({});
        if let Some(temp) = req.temperature {
            gen_cfg["temperature"] = json!(temp);
        }
        if let Some(max) = req.max_tokens {
            gen_cfg["maxOutputTokens"] = json!(max);
        }
        if let Some(ns) = req.provider_options.get("gemini") {
            merge_json_object(&mut gen_cfg, ns);
        }
        if gen_cfg.as_object().map(|m| !m.is_empty()).unwrap_or(false) {
            body["generationConfig"] = gen_cfg;
        }
        body
    }

    fn apply_auth(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(key) = &self.api_key {
            builder.header("x-goog-api-key", key)
        } else {
            builder
        }
    }
}

pub fn parse_gemini_sse_payload(payload: &str, is_interactions: bool) -> Option<AIEvent> {
    if payload == "[DONE]" {
        return Some(AIEvent::Done {
            finish_reason: "stop".to_string(),
            provider_meta: Value::Null,
        });
    }
    let v: Value = serde_json::from_str(payload).ok()?;

    if is_interactions {
        let ev_type = v.get("event_type").and_then(|t| t.as_str()).unwrap_or("");
        match ev_type {
            "content.delta" => {
                if let Some(text) = v.pointer("/delta/text").and_then(|t| t.as_str()) {
                    if !text.is_empty() {
                        return Some(AIEvent::TextDelta {
                            delta: text.to_string(),
                        });
                    }
                }
                if let Some(thought) = v.pointer("/delta/thought").and_then(|t| t.as_str()) {
                    if !thought.is_empty() {
                        return Some(AIEvent::ReasoningDelta {
                            delta: thought.to_string(),
                        });
                    }
                }
            }
            "interaction.complete" => {
                return Some(AIEvent::Done {
                    finish_reason: "completed".to_string(),
                    provider_meta: v,
                });
            }
            _ => {}
        }
        return None;
    }

    if let Some(text) = v
        .pointer("/candidates/0/content/parts/0/text")
        .and_then(|t| t.as_str())
    {
        if !text.is_empty() {
            return Some(AIEvent::TextDelta {
                delta: text.to_string(),
            });
        }
    }
    None
}

#[async_trait::async_trait]
impl AIProvider for GeminiAdapter {
    fn id(&self) -> &str {
        &self.config.id
    }

    fn protocol(&self) -> ProtocolKind {
        self.config.protocol.clone()
    }

    async fn list_models(&self) -> Result<Vec<AIModelInfo>, AIError> {
        let url = format!("{}/models", self.base_url());
        let resp = self.apply_auth(self.client.get(&url)).send().await?;
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
                let full_name = item.get("name")?.as_str()?;
                let model_id = full_name.strip_prefix("models/").unwrap_or(full_name).to_string();
                let display = item
                    .get("displayName")
                    .and_then(|d| d.as_str())
                    .unwrap_or(&model_id)
                    .to_string();
                Some(AIModelInfo {
                    id: format!("{}:{}", self.config.id, model_id),
                    provider_id: self.config.id.clone(),
                    model_id: model_id.clone(),
                    display_name: display,
                    capabilities: self.get_capabilities(&model_id),
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
            context_window: Some(1_000_000),
            max_output_tokens: Some(65_536),
        }
    }

    async fn generate(&self, req: AIRequest) -> Result<AIResponse, AIError> {
        let is_interactions = self.config.protocol == ProtocolKind::GeminiInteractions;
        let url = if is_interactions {
            format!("{}/interactions", self.base_url())
        } else {
            format!("{}/models/{}:generateContent", self.base_url(), req.model_id)
        };
        let body = if is_interactions {
            self.build_interactions_body(&req, false)
        } else {
            self.build_generate_content_body(&req)
        };

        let resp = self
            .apply_auth(self.client.post(&url))
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
        let text = if is_interactions {
            v.get("outputs")
                .and_then(|o| o.as_array())
                .and_then(|arr| arr.last())
                .and_then(|o| o.get("text"))
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string()
        } else {
            v.pointer("/candidates/0/content/parts/0/text")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string()
        };

        Ok(AIResponse {
            text,
            reasoning: None,
            input_tokens: v
                .pointer("/usageMetadata/promptTokenCount")
                .and_then(|x| x.as_u64())
                .map(|x| x as u32),
            output_tokens: v
                .pointer("/usageMetadata/candidatesTokenCount")
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
        let is_interactions = self.config.protocol == ProtocolKind::GeminiInteractions;
        let url = if is_interactions {
            format!("{}/interactions", self.base_url())
        } else {
            format!(
                "{}/models/{}:streamGenerateContent?alt=sse",
                self.base_url(),
                req.model_id
            )
        };
        let body = if is_interactions {
            self.build_interactions_body(&req, true)
        } else {
            self.build_generate_content_body(&req)
        };

        let resp = self
            .apply_auth(self.client.post(&url))
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
            if let Some(ev) = parse_gemini_sse_payload(payload, is_interactions) {
                let is_done = matches!(ev, AIEvent::Done { .. });
                let _ = chan.send(ev);
                return Ok(is_done);
            }
            Ok(false)
        })
        .await?;

        if !is_interactions {
            let _ = tx.send(AIEvent::Done {
                finish_reason: "stop".to_string(),
                provider_meta: Value::Null,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::AIMessage;

    #[test]
    fn test_gemini_interactions_and_generate_content() {
        let cfg = AIProviderConfig {
            id: "gem".into(),
            name: "Gemini".into(),
            protocol: ProtocolKind::GeminiInteractions,
            base_url: None,
            secret_ref: None,
            extra_headers: None,
            timeout_ms: 30_000,
            max_retries: 2,
            is_local: false,
            enabled: true,
        };
        let adapter = GeminiAdapter::new(cfg, Some("AIza-test".into())).unwrap();
        let req = AIRequest {
            model_id: "gemini-2.5-pro".into(),
            system: Some("Novel assistant".into()),
            messages: vec![AIMessage {
                role: "user".into(),
                content: "Check timeline".into(),
            }],
            temperature: None,
            max_tokens: None,
            tools: vec![],
            provider_options: json!({
                "gemini_interactions": {
                    "previous_interaction_id": "int_abc"
                }
            }),
        };
        let body = adapter.build_interactions_body(&req, true);
        assert_eq!(body["previous_interaction_id"], "int_abc");

        let ev = parse_gemini_sse_payload(
            r#"{"event_type":"content.delta","delta":{"text":"관측소"}}"#,
            true,
        );
        assert!(matches!(ev, Some(AIEvent::TextDelta { delta }) if delta == "관측소"));
    }
}
