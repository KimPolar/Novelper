use crate::ai::{AIError, AIEvent, AIProviderConfig};
use futures_util::StreamExt;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::time::Duration;
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

pub fn build_http_client(config: &AIProviderConfig) -> Result<reqwest::Client, AIError> {
    let mut headers = HeaderMap::new();
    if let Some(serde_json::Value::Object(map)) = &config.extra_headers {
        for (k, v) in map {
            if let Some(val_str) = v.as_str() {
                if let (Ok(name), Ok(val)) = (
                    HeaderName::from_bytes(k.as_bytes()),
                    HeaderValue::from_str(val_str),
                ) {
                    headers.insert(name, val);
                }
            }
        }
    }

    reqwest::Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_millis(config.timeout_ms.max(5_000)))
        .build()
        .map_err(AIError::from)
}

pub fn is_retryable_status(status: u16) -> bool {
    status == 429 || (500..=599).contains(&status)
}

pub fn merge_json_object(target: &mut serde_json::Value, patch: &serde_json::Value) {
    if let (Some(t_map), Some(p_map)) = (target.as_object_mut(), patch.as_object()) {
        for (k, v) in p_map {
            t_map.insert(k.clone(), v.clone());
        }
    }
}

pub async fn stream_sse_lines<F>(
    response: reqwest::Response,
    tx: &Channel<AIEvent>,
    cancel: &CancellationToken,
    mut on_data: F,
) -> Result<(), AIError>
where
    F: FnMut(&str, &Channel<AIEvent>) -> Result<bool, AIError>,
{
    let mut byte_stream = response.bytes_stream();
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
                        let text = String::from_utf8_lossy(&bytes);
                        buffer.push_str(&text);
                        while let Some(pos) = buffer.find('\n') {
                            let line = buffer[..pos].trim_end_matches('\r').to_string();
                            buffer.drain(..=pos);
                            if let Some(data) = line.strip_prefix("data:") {
                                let payload = data.trim();
                                if !payload.is_empty() {
                                    let should_stop = on_data(payload, tx)?;
                                    if should_stop {
                                        return Ok(());
                                    }
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
