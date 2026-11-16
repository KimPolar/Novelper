use crate::ai::manager::AIProviderManager;
use crate::ai::{
    repo as ai_repo, secrets, AIError, AIEvent, AIModelInfo, AIProviderConfig, AIRequest,
    ModelCapabilities, ProtocolKind, TaskBinding,
};
use crate::db::{proposal_repo, AppState, DbError};
use crate::domain::DocumentDetail;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tauri::ipc::Channel;
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveProviderInput {
    pub id: Option<String>,
    pub name: String,
    pub protocol: ProtocolKind,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub extra_headers: Option<serde_json::Value>,
    pub timeout_ms: Option<u64>,
    pub max_retries: Option<u32>,
    pub is_local: bool,
    pub enabled: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionTestResult {
    pub ok: bool,
    pub latency_ms: u128,
    pub model_count: usize,
    pub message: String,
}

#[tauri::command]
pub fn list_ai_providers(state: State<'_, AppState>) -> Result<Vec<AIProviderConfig>, String> {
    let conn = state.app_db.lock().map_err(|e| e.to_string())?;
    ai_repo::list_providers(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_ai_provider(
    state: State<'_, AppState>,
    input: SaveProviderInput,
) -> Result<AIProviderConfig, String> {
    let id = input.id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let secret_ref = if input.is_local && input.api_key.as_deref().unwrap_or("").is_empty() {
        None
    } else {
        Some(format!("provider:{}", id))
    };

    if let (Some(ref sref), Some(ref key)) = (&secret_ref, &input.api_key) {
        if !key.trim().is_empty() {
            secrets::store_api_key(sref, key.trim())?;
        }
    }

    let cfg = AIProviderConfig {
        id,
        name: input.name,
        protocol: input.protocol,
        base_url: input.base_url,
        secret_ref,
        extra_headers: input.extra_headers,
        timeout_ms: input.timeout_ms.unwrap_or(60_000),
        max_retries: input.max_retries.unwrap_or(2),
        is_local: input.is_local,
        enabled: input.enabled,
    };

    let conn = state.app_db.lock().map_err(|e| e.to_string())?;
    ai_repo::upsert_provider(&conn, &cfg).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_ai_provider(state: State<'_, AppState>, provider_id: String) -> Result<(), String> {
    let _ = secrets::delete_api_key(&format!("provider:{}", provider_id));
    let conn = state.app_db.lock().map_err(|e| e.to_string())?;
    ai_repo::delete_provider(&conn, &provider_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn test_ai_provider_connection(
    state: State<'_, AppState>,
    ai_mgr: State<'_, AIProviderManager>,
    provider_id: String,
) -> Result<ConnectionTestResult, String> {
    let cfg = {
        let conn = state.app_db.lock().map_err(|e| e.to_string())?;
        ai_repo::list_providers(&conn)
            .map_err(|e| e.to_string())?
            .into_iter()
            .find(|p| p.id == provider_id)
            .ok_or_else(|| "Provider not found".to_string())?
    };

    let api_key = cfg
        .secret_ref
        .as_deref()
        .and_then(|s| secrets::load_api_key(s).ok().flatten());
    let provider = ai_mgr
        .create_provider(&cfg, api_key)
        .map_err(|e| e.to_string())?;

    let start = Instant::now();
    match provider.list_models().await {
        Ok(models) => Ok(ConnectionTestResult {
            ok: true,
            latency_ms: start.elapsed().as_millis(),
            model_count: models.len(),
            message: format!("Connected ({} models)", models.len()),
        }),
        Err(e) => Ok(ConnectionTestResult {
            ok: false,
            latency_ms: start.elapsed().as_millis(),
            model_count: 0,
            message: e.to_string(),
        }),
    }
}

#[tauri::command]
pub async fn sync_provider_models(
    state: State<'_, AppState>,
    ai_mgr: State<'_, AIProviderManager>,
    provider_id: String,
) -> Result<Vec<AIModelInfo>, String> {
    let cfg = {
        let conn = state.app_db.lock().map_err(|e| e.to_string())?;
        ai_repo::list_providers(&conn)
            .map_err(|e| e.to_string())?
            .into_iter()
            .find(|p| p.id == provider_id)
            .ok_or_else(|| "Provider not found".to_string())?
    };

    let api_key = cfg
        .secret_ref
        .as_deref()
        .and_then(|s| secrets::load_api_key(s).ok().flatten());
    let provider = ai_mgr
        .create_provider(&cfg, api_key)
        .map_err(|e| e.to_string())?;
    let fetched = provider.list_models().await.map_err(|e| e.to_string())?;

    let conn = state.app_db.lock().map_err(|e| e.to_string())?;
    for m in &fetched {
        ai_repo::upsert_model(&conn, m).map_err(|e| e.to_string())?;
    }
    ai_repo::list_models(&conn, Some(&provider_id)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_custom_model(
    state: State<'_, AppState>,
    provider_id: String,
    model_id: String,
    display_name: String,
    capabilities: Option<ModelCapabilities>,
) -> Result<AIModelInfo, String> {
    let model = AIModelInfo {
        id: format!("{}:{}", provider_id, model_id),
        provider_id,
        model_id,
        display_name,
        capabilities: capabilities.unwrap_or_default(),
        default_options: None,
        is_custom: true,
    };
    let conn = state.app_db.lock().map_err(|e| e.to_string())?;
    ai_repo::upsert_model(&conn, &model).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_ai_models(
    state: State<'_, AppState>,
    provider_id: Option<String>,
) -> Result<Vec<AIModelInfo>, String> {
    let conn = state.app_db.lock().map_err(|e| e.to_string())?;
    ai_repo::list_models(&conn, provider_id.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_ai_model(state: State<'_, AppState>, model_id: String) -> Result<(), String> {
    let conn = state.app_db.lock().map_err(|e| e.to_string())?;
    ai_repo::delete_model(&conn, &model_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_task_bindings(state: State<'_, AppState>) -> Result<Vec<TaskBinding>, String> {
    let conn = state.app_db.lock().map_err(|e| e.to_string())?;
    ai_repo::list_task_bindings(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_task_binding(
    state: State<'_, AppState>,
    binding: TaskBinding,
) -> Result<(), String> {
    let conn = state.app_db.lock().map_err(|e| e.to_string())?;
    ai_repo::upsert_task_binding(&conn, &binding).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stream_ai_chat(
    state: State<'_, AppState>,
    ai_mgr: State<'_, AIProviderManager>,
    request_id: String,
    model_record_id: String,
    task_type: String,
    mut request: AIRequest,
    on_event: Channel<AIEvent>,
) -> Result<(), String> {
    let (cfg, model_info) = {
        let conn = state.app_db.lock().map_err(|e| e.to_string())?;
        ai_repo::find_provider_and_model(&conn, &model_record_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Selected model or provider not found".to_string())?
    };

    request.model_id = model_info.model_id.clone();
    let api_key = cfg
        .secret_ref
        .as_deref()
        .and_then(|s| secrets::load_api_key(s).ok().flatten());
    let provider = ai_mgr
        .create_provider(&cfg, api_key)
        .map_err(|e| e.to_string())?;

    let cancel = ai_mgr.register_cancel_token(&request_id);
    let start = Instant::now();
    let res = provider.stream(request, on_event.clone(), cancel).await;
    ai_mgr.finish_request(&request_id);

    let status_str = match &res {
        Ok(()) => "success",
        Err(AIError::Cancelled) => "cancelled",
        Err(_) => "error",
    };

    if let Ok(active) = state.active_project.lock() {
        if let Some(proj) = active.as_ref() {
            let _ = ai_repo::log_ai_request(
                &proj.conn,
                &proj.project_id,
                &cfg.id,
                &model_info.model_id,
                &task_type,
                None,
                None,
                start.elapsed().as_millis() as i64,
                status_str,
            );
        }
    }

    res.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cancel_ai_request(ai_mgr: State<'_, AIProviderManager>, request_id: String) -> bool {
    ai_mgr.cancel_request(&request_id)
}

#[tauri::command]
pub fn create_ai_proposal(
    state: State<'_, AppState>,
    doc_id: String,
    target_block_id: String,
    original_text: String,
    proposed_text: String,
    rationale: Option<String>,
) -> Result<proposal_repo::AIProposal, String> {
    let active = state.active_project.lock().map_err(|e| e.to_string())?;
    let proj = active
        .as_ref()
        .ok_or_else(|| DbError::NoActiveProject.to_string())?;
    proposal_repo::create_proposal(
        &proj.conn,
        &doc_id,
        &target_block_id,
        &original_text,
        &proposed_text,
        rationale,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_ai_proposals(
    state: State<'_, AppState>,
    doc_id: String,
) -> Result<Vec<proposal_repo::AIProposal>, String> {
    let active = state.active_project.lock().map_err(|e| e.to_string())?;
    let proj = active
        .as_ref()
        .ok_or_else(|| DbError::NoActiveProject.to_string())?;
    proposal_repo::list_proposals(&proj.conn, &doc_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn apply_ai_proposal(
    state: State<'_, AppState>,
    proposal_id: String,
) -> Result<DocumentDetail, String> {
    let active = state.active_project.lock().map_err(|e| e.to_string())?;
    let proj = active
        .as_ref()
        .ok_or_else(|| DbError::NoActiveProject.to_string())?;
    proposal_repo::apply_proposal(&proj.conn, &proposal_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reject_ai_proposal(
    state: State<'_, AppState>,
    proposal_id: String,
) -> Result<(), String> {
    let active = state.active_project.lock().map_err(|e| e.to_string())?;
    let proj = active
        .as_ref()
        .ok_or_else(|| DbError::NoActiveProject.to_string())?;
    proposal_repo::reject_proposal(&proj.conn, &proposal_id).map_err(|e| e.to_string())
}
