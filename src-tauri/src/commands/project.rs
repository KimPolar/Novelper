use crate::db::{
    open_project_connection, project_repo, version_repo, ActiveProject, AppState, DbError,
};
use crate::domain::{
    CreateDocumentInput, CreateProjectInput, DiffLine, DocumentDetail, DocumentTreeItem,
    DocumentVersionItem, FtsSearchHit, ProjectDetail, ProjectStats, ProjectSummary, ReorderItem,
    UpdateDocumentInput,
};
use std::path::PathBuf;
use tauri::State;

fn map_err(err: DbError) -> String {
    err.to_string()
}

#[tauri::command]
pub fn list_projects(state: State<'_, AppState>) -> Result<Vec<ProjectSummary>, String> {
    let app_conn = state.app_db.lock().map_err(|e| e.to_string())?;
    project_repo::list_projects_from_app_db(&app_conn).map_err(map_err)
}

#[tauri::command]
pub fn create_project(
    state: State<'_, AppState>,
    input: CreateProjectInput,
) -> Result<ProjectDetail, String> {
    let temp_id = uuid::Uuid::new_v4().to_string();
    let db_path = state.projects_dir().join(&temp_id).join("project.db");
    let proj_conn = open_project_connection(&db_path).map_err(map_err)?;
    let detail = project_repo::create_project(&proj_conn, input).map_err(map_err)?;

    let db_path_str = db_path.to_string_lossy().to_string();
    {
        let app_conn = state.app_db.lock().map_err(|e| e.to_string())?;
        project_repo::register_project_in_app_db(&app_conn, &detail, &db_path_str)
            .map_err(map_err)?;
    }

    let mut active = state.active_project.lock().map_err(|e| e.to_string())?;
    *active = Some(ActiveProject {
        project_id: detail.id.clone(),
        db_path,
        conn: proj_conn,
    });

    Ok(detail)
}

#[tauri::command]
pub fn open_project(state: State<'_, AppState>, db_path: String) -> Result<ProjectDetail, String> {
    let path = PathBuf::from(&db_path);
    let proj_conn = open_project_connection(&path).map_err(map_err)?;
    let detail = project_repo::get_project_detail(&proj_conn).map_err(map_err)?;

    {
        let app_conn = state.app_db.lock().map_err(|e| e.to_string())?;
        project_repo::register_project_in_app_db(&app_conn, &detail, &db_path).map_err(map_err)?;
    }

    let mut active = state.active_project.lock().map_err(|e| e.to_string())?;
    *active = Some(ActiveProject {
        project_id: detail.id.clone(),
        db_path: path,
        conn: proj_conn,
    });

    Ok(detail)
}

#[tauri::command]
pub fn list_documents(state: State<'_, AppState>) -> Result<Vec<DocumentTreeItem>, String> {
    let active = state.active_project.lock().map_err(|e| e.to_string())?;
    let proj = active
        .as_ref()
        .ok_or_else(|| DbError::NoActiveProject.to_string())?;
    project_repo::list_documents(&proj.conn, &proj.project_id).map_err(map_err)
}

#[tauri::command]
pub fn get_document(state: State<'_, AppState>, doc_id: String) -> Result<DocumentDetail, String> {
    let active = state.active_project.lock().map_err(|e| e.to_string())?;
    let proj = active
        .as_ref()
        .ok_or_else(|| DbError::NoActiveProject.to_string())?;
    project_repo::get_document(&proj.conn, &doc_id).map_err(map_err)
}

#[tauri::command]
pub fn create_document(
    state: State<'_, AppState>,
    input: CreateDocumentInput,
) -> Result<DocumentDetail, String> {
    let active = state.active_project.lock().map_err(|e| e.to_string())?;
    let proj = active
        .as_ref()
        .ok_or_else(|| DbError::NoActiveProject.to_string())?;
    project_repo::create_document(&proj.conn, &proj.project_id, input).map_err(map_err)
}

#[tauri::command]
pub fn update_document(
    state: State<'_, AppState>,
    input: UpdateDocumentInput,
) -> Result<DocumentDetail, String> {
    let active = state.active_project.lock().map_err(|e| e.to_string())?;
    let proj = active
        .as_ref()
        .ok_or_else(|| DbError::NoActiveProject.to_string())?;
    project_repo::update_document(&proj.conn, input).map_err(map_err)
}

#[tauri::command]
pub fn reorder_documents(
    state: State<'_, AppState>,
    items: Vec<ReorderItem>,
) -> Result<(), String> {
    let active = state.active_project.lock().map_err(|e| e.to_string())?;
    let proj = active
        .as_ref()
        .ok_or_else(|| DbError::NoActiveProject.to_string())?;
    project_repo::reorder_documents(&proj.conn, &items).map_err(map_err)
}

#[tauri::command]
pub fn set_document_trashed(
    state: State<'_, AppState>,
    doc_id: String,
    trashed: bool,
) -> Result<(), String> {
    let active = state.active_project.lock().map_err(|e| e.to_string())?;
    let proj = active
        .as_ref()
        .ok_or_else(|| DbError::NoActiveProject.to_string())?;
    project_repo::set_document_trashed(&proj.conn, &doc_id, trashed).map_err(map_err)
}

#[tauri::command]
pub fn search_documents_fts(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<FtsSearchHit>, String> {
    let active = state.active_project.lock().map_err(|e| e.to_string())?;
    let proj = active
        .as_ref()
        .ok_or_else(|| DbError::NoActiveProject.to_string())?;
    project_repo::search_documents_fts(&proj.conn, &query).map_err(map_err)
}

#[tauri::command]
pub fn create_document_version(
    state: State<'_, AppState>,
    doc_id: String,
    snapshot_type: String,
    note: Option<String>,
) -> Result<DocumentVersionItem, String> {
    let active = state.active_project.lock().map_err(|e| e.to_string())?;
    let proj = active
        .as_ref()
        .ok_or_else(|| DbError::NoActiveProject.to_string())?;
    version_repo::create_document_version(&proj.conn, &doc_id, &snapshot_type, note).map_err(map_err)
}

#[tauri::command]
pub fn list_document_versions(
    state: State<'_, AppState>,
    doc_id: String,
) -> Result<Vec<DocumentVersionItem>, String> {
    let active = state.active_project.lock().map_err(|e| e.to_string())?;
    let proj = active
        .as_ref()
        .ok_or_else(|| DbError::NoActiveProject.to_string())?;
    version_repo::list_document_versions(&proj.conn, &doc_id).map_err(map_err)
}

#[tauri::command]
pub fn restore_document_version(
    state: State<'_, AppState>,
    version_id: String,
) -> Result<DocumentDetail, String> {
    let active = state.active_project.lock().map_err(|e| e.to_string())?;
    let proj = active
        .as_ref()
        .ok_or_else(|| DbError::NoActiveProject.to_string())?;
    version_repo::restore_document_version(&proj.conn, &version_id).map_err(map_err)
}

#[tauri::command]
pub fn compute_line_diff(old_text: String, new_text: String) -> Vec<DiffLine> {
    version_repo::compute_line_diff(&old_text, &new_text)
}

#[tauri::command]
pub fn get_project_stats(state: State<'_, AppState>) -> Result<ProjectStats, String> {
    let active = state.active_project.lock().map_err(|e| e.to_string())?;
    let proj = active
        .as_ref()
        .ok_or_else(|| DbError::NoActiveProject.to_string())?;
    version_repo::get_project_stats(&proj.conn, &proj.project_id).map_err(map_err)
}
