pub mod ai;
pub mod commands;
pub mod context;
pub mod db;
pub mod domain;

use ai::manager::AIProviderManager;
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
            app.manage(AIProviderManager::new());
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
            commands::ai::list_ai_providers,
            commands::ai::save_ai_provider,
            commands::ai::delete_ai_provider,
            commands::ai::test_ai_provider_connection,
            commands::ai::sync_provider_models,
            commands::ai::add_custom_model,
            commands::ai::list_ai_models,
            commands::ai::delete_ai_model,
            commands::ai::list_task_bindings,
            commands::ai::save_task_binding,
            commands::ai::stream_ai_chat,
            commands::ai::cancel_ai_request,
            commands::ai::create_ai_proposal,
            commands::ai::list_ai_proposals,
            commands::ai::apply_ai_proposal,
            commands::ai::reject_ai_proposal,
            commands::world::list_characters,
            commands::world::upsert_character,
            commands::world::list_locations,
            commands::world::upsert_location,
            commands::world::list_worldbuilding,
            commands::world::upsert_worldbuilding,
            commands::world::list_timeline_events,
            commands::world::upsert_timeline_event,
            commands::world::list_facts,
            commands::world::add_fact,
            commands::world::build_scene_context_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
