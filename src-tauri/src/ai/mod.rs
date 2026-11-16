pub mod adapters;
pub mod manager;
pub mod repo;
pub mod secrets;

use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolKind {
    OpenaiChat,
    OpenaiResponses,
    AnthropicMessages,
    GeminiInteractions,
    GeminiGenerateContent,
    OllamaNative,
    OpenaiCompat,
}

impl ProtocolKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OpenaiChat => "openai_chat",
            Self::OpenaiResponses => "openai_responses",
            Self::AnthropicMessages => "anthropic_messages",
            Self::GeminiInteractions => "gemini_interactions",
            Self::GeminiGenerateContent => "gemini_generate_content",
            Self::OllamaNative => "ollama_native",
            Self::OpenaiCompat => "openai_compat",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "openai_chat" => Some(Self::OpenaiChat),
            "openai_responses" => Some(Self::OpenaiResponses),
            "anthropic_messages" => Some(Self::AnthropicMessages),
            "gemini_interactions" => Some(Self::GeminiInteractions),
            "gemini_generate_content" => Some(Self::GeminiGenerateContent),
            "ollama_native" => Some(Self::OllamaNative),
            "openai_compat" => Some(Self::OpenaiCompat),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCapabilities {
    pub streaming: bool,
    pub vision: bool,
    pub tool_calling: bool,
    pub structured_output: bool,
    pub reasoning: bool,
    pub prompt_caching: bool,
    pub context_window: Option<u32>,
    pub max_output_tokens: Option<u32>,
}

impl Default for ModelCapabilities {
    fn default() -> Self {
        Self {
            streaming: true,
            vision: false,
            tool_calling: true,
            structured_output: true,
            reasoning: false,
            prompt_caching: false,
            context_window: Some(128_000),
            max_output_tokens: Some(8_192),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AIModelInfo {
    pub id: String,
    pub provider_id: String,
    pub model_id: String,
    pub display_name: String,
    pub capabilities: ModelCapabilities,
    pub default_options: Option<serde_json::Value>,
    pub is_custom: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AIProviderConfig {
    pub id: String,
    pub name: String,
    pub protocol: ProtocolKind,
    pub base_url: Option<String>,
    pub secret_ref: Option<String>,
    pub extra_headers: Option<serde_json::Value>,
    pub timeout_ms: u64,
    pub max_retries: u32,
    pub is_local: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskBinding {
    pub task_type: String,
    pub primary_model_id: Option<String>,
    pub fallback_model_ids: Vec<String>,
    pub allow_external_fallback: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AIMessage {
    pub role: String, // "user" | "assistant" | "system" | "tool"
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters_schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AIRequest {
    pub model_id: String,
    pub system: Option<String>,
    pub messages: Vec<AIMessage>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub tools: Vec<ToolDefinition>,
    #[serde(default)]
    pub provider_options: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AIResponse {
    pub text: String,
    pub reasoning: Option<String>,
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    pub finish_reason: String,
    pub provider_meta: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AIEvent {
    TextDelta { delta: String },
    ReasoningDelta { delta: String },
    ToolCall { id: String, name: String, arguments_json: String },
    Usage { input_tokens: u32, output_tokens: u32 },
    Done { finish_reason: String, provider_meta: serde_json::Value },
    Error { code: String, message: String, retryable: bool },
}

#[derive(Debug, thiserror::Error)]
pub enum AIError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Provider error ({status}): {message}")]
    Provider { status: u16, message: String, retryable: bool },
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Request cancelled")]
    Cancelled,
}

#[async_trait::async_trait]
pub trait AIProvider: Send + Sync {
    fn id(&self) -> &str;
    fn protocol(&self) -> ProtocolKind;
    async fn list_models(&self) -> Result<Vec<AIModelInfo>, AIError>;
    fn get_capabilities(&self, model_id: &str) -> ModelCapabilities;
    async fn generate(&self, req: AIRequest) -> Result<AIResponse, AIError>;
    async fn stream(
        &self,
        req: AIRequest,
        tx: Channel<AIEvent>,
        cancel: CancellationToken,
    ) -> Result<(), AIError>;
}
