use crate::ai::{AIError, AIProvider, AIProviderConfig};

pub fn build_adapter(
    config: &AIProviderConfig,
    _api_key: Option<String>,
) -> Result<Box<dyn AIProvider>, AIError> {
    Err(AIError::Config(format!(
        "Adapter for {:?} will be registered in Phase 2-B/2-C",
        config.protocol
    )))
}
