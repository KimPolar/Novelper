use crate::ai::adapters::build_adapter;
use crate::ai::{AIError, AIProvider, AIProviderConfig};
use std::collections::HashMap;
use std::sync::Mutex;
use tokio_util::sync::CancellationToken;

pub struct AIProviderManager {
    active_requests: Mutex<HashMap<String, CancellationToken>>,
}

impl AIProviderManager {
    pub fn new() -> Self {
        Self {
            active_requests: Mutex::new(HashMap::new()),
        }
    }

    pub fn register_cancel_token(&self, request_id: &str) -> CancellationToken {
        let token = CancellationToken::new();
        if let Ok(mut map) = self.active_requests.lock() {
            map.insert(request_id.to_string(), token.clone());
        }
        token
    }

    pub fn cancel_request(&self, request_id: &str) -> bool {
        if let Ok(mut map) = self.active_requests.lock() {
            if let Some(token) = map.remove(request_id) {
                token.cancel();
                return true;
            }
        }
        false
    }

    pub fn finish_request(&self, request_id: &str) {
        if let Ok(mut map) = self.active_requests.lock() {
            map.remove(request_id);
        }
    }

    pub fn create_provider(
        &self,
        config: &AIProviderConfig,
        api_key: Option<String>,
    ) -> Result<Box<dyn AIProvider>, AIError> {
        build_adapter(config, api_key)
    }
}

impl Default for AIProviderManager {
    fn default() -> Self {
        Self::new()
    }
}
