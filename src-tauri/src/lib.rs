pub mod ai;
pub mod commands;
pub mod db;
pub mod domain;

use db::AppState;
use tauri::Manager;

#[tauri::command]
fn app_info() -> serde_json::Value {
    serde_json::json!({
        "name": "novel-studio",
        "version": env!("CARGO_PKG_VERSION")
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            let state = AppState::init(data_dir).expect("failed to initialize app database");
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_info,
            commands::project::list_projects,
            commands::project::create_project,
            commands::project::open_project,
            commands::project::list_documents,
            commands::project::get_document,
            commands::project::create_document,
            commands::project::update_document,
            commands::project::reorder_documents,
            commands::project::set_document_trashed,
            commands::project::search_documents_fts,
            commands::project::create_document_version,
            commands::project::list_document_versions,
            commands::project::restore_document_version,
            commands::project::compute_line_diff,
            commands::project::get_project_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
