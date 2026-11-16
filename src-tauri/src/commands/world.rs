use crate::context::{build_scene_context, BuiltContext};
use crate::db::{world_repo, AppState, DbError};
use tauri::State;

#[tauri::command]
pub fn list_characters(
    state: State<'_, AppState>,
) -> Result<Vec<world_repo::CharacterItem>, String> {
    let active = state.active_project.lock().map_err(|e| e.to_string())?;
    let proj = active
        .as_ref()
        .ok_or_else(|| DbError::NoActiveProject.to_string())?;
    world_repo::list_characters(&proj.conn, &proj.project_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn upsert_character(
    state: State<'_, AppState>,
    id: Option<String>,
    name: String,
    role: Option<String>,
    profile_json: Option<String>,
) -> Result<world_repo::CharacterItem, String> {
    let active = state.active_project.lock().map_err(|e| e.to_string())?;
    let proj = active
        .as_ref()
        .ok_or_else(|| DbError::NoActiveProject.to_string())?;
    world_repo::upsert_character(&proj.conn, &proj.project_id, id, name, role, profile_json)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_locations(
    state: State<'_, AppState>,
) -> Result<Vec<world_repo::LocationItem>, String> {
    let active = state.active_project.lock().map_err(|e| e.to_string())?;
    let proj = active
        .as_ref()
        .ok_or_else(|| DbError::NoActiveProject.to_string())?;
    world_repo::list_locations(&proj.conn, &proj.project_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn upsert_location(
    state: State<'_, AppState>,
    id: Option<String>,
    name: String,
    description: Option<String>,
) -> Result<world_repo::LocationItem, String> {
    let active = state.active_project.lock().map_err(|e| e.to_string())?;
    let proj = active
        .as_ref()
        .ok_or_else(|| DbError::NoActiveProject.to_string())?;
    world_repo::upsert_location(&proj.conn, &proj.project_id, id, name, description)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_worldbuilding(
    state: State<'_, AppState>,
) -> Result<Vec<world_repo::WorldbuildingItem>, String> {
    let active = state.active_project.lock().map_err(|e| e.to_string())?;
    let proj = active
        .as_ref()
        .ok_or_else(|| DbError::NoActiveProject.to_string())?;
    world_repo::list_worldbuilding(&proj.conn, &proj.project_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn upsert_worldbuilding(
    state: State<'_, AppState>,
    id: Option<String>,
    category: String,
    title: String,
    content: String,
) -> Result<world_repo::WorldbuildingItem, String> {
    let active = state.active_project.lock().map_err(|e| e.to_string())?;
    let proj = active
        .as_ref()
        .ok_or_else(|| DbError::NoActiveProject.to_string())?;
    world_repo::upsert_worldbuilding(&proj.conn, &proj.project_id, id, category, title, content)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_timeline_events(
    state: State<'_, AppState>,
) -> Result<Vec<world_repo::TimelineEventItem>, String> {
    let active = state.active_project.lock().map_err(|e| e.to_string())?;
    let proj = active
        .as_ref()
        .ok_or_else(|| DbError::NoActiveProject.to_string())?;
    world_repo::list_timeline_events(&proj.conn, &proj.project_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn upsert_timeline_event(
    state: State<'_, AppState>,
    id: Option<String>,
    document_id: Option<String>,
    sequence_index: i64,
    in_story_time: Option<String>,
    title: String,
    summary: Option<String>,
) -> Result<world_repo::TimelineEventItem, String> {
    let active = state.active_project.lock().map_err(|e| e.to_string())?;
    let proj = active
        .as_ref()
        .ok_or_else(|| DbError::NoActiveProject.to_string())?;
    world_repo::upsert_timeline_event(
        &proj.conn,
        &proj.project_id,
        id,
        document_id,
        sequence_index,
        in_story_time,
        title,
        summary,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_facts(state: State<'_, AppState>) -> Result<Vec<world_repo::FactItem>, String> {
    let active = state.active_project.lock().map_err(|e| e.to_string())?;
    let proj = active
        .as_ref()
        .ok_or_else(|| DbError::NoActiveProject.to_string())?;
    world_repo::list_facts(&proj.conn, &proj.project_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_fact(
    state: State<'_, AppState>,
    subject_type: String,
    subject_id: String,
    fact_text: String,
    certainty: String,
    evidence_doc_id: Option<String>,
    evidence_block_id: Option<String>,
    evidence_quote: Option<String>,
) -> Result<world_repo::FactItem, String> {
    let active = state.active_project.lock().map_err(|e| e.to_string())?;
    let proj = active
        .as_ref()
        .ok_or_else(|| DbError::NoActiveProject.to_string())?;
    world_repo::add_fact(
        &proj.conn,
        &proj.project_id,
        subject_type,
        subject_id,
        fact_text,
        certainty,
        evidence_doc_id,
        evidence_block_id,
        evidence_quote,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn build_scene_context_cmd(
    state: State<'_, AppState>,
    doc_id: Option<String>,
    query: Option<String>,
    token_budget: Option<usize>,
) -> Result<BuiltContext, String> {
    let active = state.active_project.lock().map_err(|e| e.to_string())?;
    let proj = active
        .as_ref()
        .ok_or_else(|| DbError::NoActiveProject.to_string())?;
    build_scene_context(
        &proj.conn,
        &proj.project_id,
        doc_id.as_deref(),
        query.as_deref(),
        token_budget.unwrap_or(6000),
    )
    .map_err(|e| e.to_string())
}
