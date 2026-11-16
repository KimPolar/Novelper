use crate::db::{AppState, DbError};
use crate::export;
use std::path::PathBuf;
use tauri::State;

#[tauri::command]
pub fn export_manuscript(
    state: State<'_, AppState>,
    format: String,
    output_dir: Option<String>,
) -> Result<String, String> {
    let active = state.active_project.lock().map_err(|e| e.to_string())?;
    let proj = active
        .as_ref()
        .ok_or_else(|| DbError::NoActiveProject.to_string())?;

    let base_dir = output_dir
        .map(PathBuf::from)
        .unwrap_or_else(|| state.data_dir.join("exports"));
    let ext = match format.as_str() {
        "docx" => "docx",
        "epub" => "epub",
        "html" | "pdf" => "html",
        _ => "md",
    };
    let out_path = base_dir.join(format!("novel_{}.{}", proj.project_id, ext));

    match format.as_str() {
        "docx" => export::export_docx(&proj.conn, &proj.project_id, &out_path),
        "epub" => export::export_epub(&proj.conn, &proj.project_id, &out_path),
        "html" | "pdf" => export::export_html_print(&proj.conn, &proj.project_id, &out_path),
        _ => export::export_markdown(&proj.conn, &proj.project_id, &out_path),
    }
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn backup_active_project(state: State<'_, AppState>) -> Result<String, String> {
    let active = state.active_project.lock().map_err(|e| e.to_string())?;
    let proj = active
        .as_ref()
        .ok_or_else(|| DbError::NoActiveProject.to_string())?;
    let backup_dir = state.data_dir.join("backups").join(&proj.project_id);
    let dest = export::create_project_backup(&proj.db_path, &backup_dir).map_err(|e| e.to_string())?;
    Ok(dest.to_string_lossy().to_string())
}
