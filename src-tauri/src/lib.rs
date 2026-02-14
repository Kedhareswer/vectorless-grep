pub mod commands;
pub mod core;
pub mod db;
pub mod providers;
pub mod reasoner;
pub mod security;
pub mod sidecar;

use std::path::PathBuf;

use tauri::Manager;

use db::{default_data_dir, Database};
use providers::gemini::GeminiClient;
use reasoner::executor::ReasoningExecutor;

fn log_level_from_env() -> tauri_plugin_log::log::LevelFilter {
    match std::env::var("VECTORLESS_LOG")
        .unwrap_or_else(|_| "info".to_string())
        .to_ascii_lowercase()
        .as_str()
    {
        "trace" => tauri_plugin_log::log::LevelFilter::Trace,
        "debug" => tauri_plugin_log::log::LevelFilter::Debug,
        "warn" => tauri_plugin_log::log::LevelFilter::Warn,
        "error" => tauri_plugin_log::log::LevelFilter::Error,
        _ => tauri_plugin_log::log::LevelFilter::Info,
    }
}

fn sqlx_debug_enabled() -> bool {
    matches!(
        std::env::var("VECTORLESS_SQLX_DEBUG")
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    )
}

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub executor: ReasoningExecutor,
    pub data_dir: PathBuf,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let log_level = log_level_from_env();
    let log_builder = tauri_plugin_log::Builder::default().level(log_level);
    let log_builder = if sqlx_debug_enabled() {
        log_builder
    } else {
        log_builder.level_for("sqlx::query", tauri_plugin_log::log::LevelFilter::Warn)
    };

    tauri::Builder::default()
        .plugin(log_builder.build())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let base_data_dir = app
                .path()
                .app_data_dir()
                .ok()
                .unwrap_or_else(|| default_data_dir(None).expect("data dir"));
            let data_dir = base_data_dir.join("vectorless");
            let db = tauri::async_runtime::block_on(Database::new(&data_dir))
                .map_err(|err| std::io::Error::other(err.to_string()))?;

            let gemini = GeminiClient::new("gemini-2.0-flash")
                .map_err(|err| std::io::Error::other(err.to_string()))?;
            let executor = ReasoningExecutor::new(gemini);
            app.manage(AppState {
                db,
                executor,
                data_dir,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings::set_provider_key,
            commands::projects::list_projects,
            commands::projects::create_project,
            commands::projects::rename_project,
            commands::projects::delete_project,
            commands::documents::ingest_document,
            commands::documents::list_documents,
            commands::documents::open_document,
            commands::documents::get_tree,
            commands::documents::get_project_tree,
            commands::documents::get_node,
            commands::documents::get_document_preview,
            commands::documents::get_graph_layout,
            commands::documents::save_graph_layout,
            commands::documents::export_markdown,
            commands::documents::delete_document,
            commands::reasoning::run_reasoning_query,
            commands::reasoning::get_run,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
