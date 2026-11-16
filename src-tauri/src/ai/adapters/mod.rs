pub mod anthropic;
pub mod common;
pub mod gemini;
pub mod openai;

use crate::ai::{AIError, AIProvider, AIProviderConfig, ProtocolKind};
use anthropic::AnthropicAdapter;
use gemini::GeminiAdapter;
use openai::OpenAIAdapter;

pub fn build_adapter(
    config: &AIProviderConfig,
    api_key: Option<String>,
) -> Result<Box<dyn AIProvider>, AIError> {
    match config.protocol {
        ProtocolKind::OpenaiChat | ProtocolKind::OpenaiResponses => {
            Ok(Box::new(OpenAIAdapter::new(config.clone(), api_key)?))
        }
        ProtocolKind::AnthropicMessages => {
            Ok(Box::new(AnthropicAdapter::new(config.clone(), api_key)?))
        }
        ProtocolKind::GeminiInteractions | ProtocolKind::GeminiGenerateContent => {
            Ok(Box::new(GeminiAdapter::new(config.clone(), api_key)?))
        }
        ProtocolKind::OllamaNative | ProtocolKind::OpenaiCompat => Err(AIError::Config(format!(
            "Local/custom adapter {:?} pending Phase 2-C",
            config.protocol
        ))),
    }
}
