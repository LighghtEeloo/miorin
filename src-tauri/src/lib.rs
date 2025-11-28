mod db;
mod watcher;
mod settings;
mod blob;

use db::*;
use watcher::*;
use settings::*;
use blob::*;

#[tauri::command]
fn toggle_devtools(window: tauri::WebviewWindow) {
    window.open_devtools();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations(
                    "sqlite:miorin.db",
                    vec![tauri_plugin_sql::Migration {
                        version: 1,
                        description: "create initial schema",
                        sql: include_str!("../migrations/001_initial_schema.sql"),
                        kind: tauri_plugin_sql::MigrationKind::Up,
                    }],
                )
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            toggle_devtools,
            get_watch_paths_cmd,
            set_watch_paths_cmd,
            add_watch_path_cmd,
            remove_watch_path_cmd,
            toggle_watch_path_enabled_cmd,
            import_files_from_path_cmd,
            get_all_raw_entries,
            create_raw_entry,
            update_raw_entry,
            delete_raw_entry,
            delete_all_raw_entries,
            get_all_cubes,
            create_cube,
            update_cube,
            delete_cube,
            store_blob_cmd,
            store_blob_from_file_cmd,
            get_blob_cmd,
            blob_exists_cmd,
            delete_blob_cmd,
            generate_thumbnail_cmd,
            get_all_store_data_cmd,
        ])
        .setup(|app| {
            // Start file watcher for configured path
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = start_file_watcher(app_handle).await {
                    tracing::error!("Failed to start file watcher: {}", e);
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
