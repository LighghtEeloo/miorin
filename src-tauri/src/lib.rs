mod db;
mod watcher;
mod settings;
mod blob;

use db::*;
use watcher::*;
use settings::*;
use blob::*;
use tauri::AppHandle;

#[tauri::command]
fn toggle_devtools(window: tauri::WebviewWindow) {
    if window.is_devtools_open() {
        window.close_devtools();
    } else {
    window.open_devtools();
    }
}

#[tauri::command]
fn set_devtools_visible(window: tauri::WebviewWindow, visible: bool) {
    if visible {
        window.open_devtools();
    } else {
        window.close_devtools();
    }
}

#[tauri::command]
fn is_devtools_open(window: tauri::WebviewWindow) -> bool {
    window.is_devtools_open()
}

/// Open a folder picker dialog and return the selected path
#[tauri::command]
async fn open_folder_dialog_cmd() -> Result<Option<String>, String> {
    use rfd::AsyncFileDialog;
    
    let default_dir = dirs::home_dir().unwrap_or_default();
    let dialog = AsyncFileDialog::new()
        .set_directory(&default_dir)
        .pick_folder()
        .await;
    
    Ok(dialog.map(|handle| handle.path().to_string_lossy().to_string()))
}

/// Reveal the app data folder in the file manager
#[tauri::command]
async fn reveal_data_folder_cmd(app: AppHandle) -> Result<String, String> {
    use tauri::Manager;
    let app_dir = app.path().app_data_dir().map_err(|e| {
        tracing::error!("Failed to get app data dir: {}", e);
        format!("Failed to get app data dir: {}", e)
    })?;
    
    // Ensure the directory exists
    std::fs::create_dir_all(&app_dir).map_err(|e| {
        tracing::error!("Failed to create app data dir: {}", e);
        format!("Failed to create app data dir: {}", e)
    })?;
    
    let path_str = app_dir.to_string_lossy().to_string();
    
    // Open the folder using platform-specific commands
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        // Canonicalize the path to ensure it's absolute and properly formatted
        let canonical_path = std::fs::canonicalize(&app_dir)
            .map_err(|e| format!("Failed to canonicalize path: {}", e))?;
        let canonical_str = canonical_path.to_string_lossy().to_string();
        
        tracing::info!(path = %canonical_str, "Opening folder in Finder");
        
        // Use -R flag to reveal the folder in Finder
        let output = Command::new("/usr/bin/open")
            .arg("-R")
            .arg(&canonical_str)
            .output()
            .map_err(|e| format!("Failed to execute open command: {}", e))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to open folder: {}", stderr));
        }
    }
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        Command::new("explorer")
            .arg(&path_str)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        Command::new("xdg-open")
            .arg(&path_str)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    Ok(path_str)
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
            set_devtools_visible,
            is_devtools_open,
            get_devtools_visible_cmd,
            set_devtools_visible_cmd,
            get_watch_paths_cmd,
            set_watch_paths_cmd,
            add_watch_path_cmd,
            remove_watch_path_cmd,
            toggle_watch_path_enabled_cmd,
            import_files_from_path_cmd,
            get_panel_left_width_cmd,
            set_panel_left_width_cmd,
            get_panel_right_width_cmd,
            set_panel_right_width_cmd,
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
            reveal_data_folder_cmd,
            open_folder_dialog_cmd,
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
