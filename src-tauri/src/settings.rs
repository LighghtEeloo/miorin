use std::path::PathBuf;
use tauri::AppHandle;
use tracing;

const SETTINGS_STORE_NAME: &str = "settings.json";
const WATCH_PATHS_KEY: &str = "watch_paths";
const ENABLE_WATCHER_KEY: &str = "enable_watcher";

/// Get the store helper function
async fn get_store(
    app: &AppHandle,
) -> Result<std::sync::Arc<tauri_plugin_store::Store<tauri::Wry>>, String> {
    use tauri_plugin_store::StoreBuilder;
    let store_path = std::path::PathBuf::from(SETTINGS_STORE_NAME);
    StoreBuilder::new(app, store_path).build().map_err(|e| format!("Failed to build store: {}", e))
}

/// Get all watch paths from settings
pub async fn get_watch_paths(app: &AppHandle) -> Result<Vec<PathBuf>, String> {
    let store = get_store(app).await?;

    if let Some(paths_value) = store.get(WATCH_PATHS_KEY) {
        if let Some(paths_array) = paths_value.as_array() {
            let paths: Vec<PathBuf> =
                paths_array.iter().filter_map(|v| v.as_str().map(PathBuf::from)).collect();
            if !paths.is_empty() {
                tracing::debug!(watch_paths = ?paths, "Found watch paths in settings");
                return Ok(paths);
            }
        }
    }

    // Return default path if no paths are configured
    let default_path =
        dirs::home_dir().ok_or("Failed to get home directory")?.join("Downloads/Pic");

    tracing::debug!(default_path = ?default_path, "Using default watch path");
    Ok(vec![default_path])
}

/// Set watch paths in settings
pub async fn set_watch_paths(app: &AppHandle, paths: Vec<PathBuf>) -> Result<(), String> {
    let path_strings: Vec<String> = paths.iter().map(|p| p.to_string_lossy().to_string()).collect();

    tracing::info!(watch_paths = ?path_strings, "Setting watch paths in settings");

    let store = get_store(app).await?;
    store.set(WATCH_PATHS_KEY.to_string(), serde_json::json!(path_strings));
    store.save().map_err(|e| format!("Failed to save settings: {}", e))?;

    Ok(())
}

/// Get whether the watcher is enabled
pub async fn get_enable_watcher(app: &AppHandle) -> Result<bool, String> {
    let store = get_store(app).await?;

    if let Some(enabled_value) = store.get(ENABLE_WATCHER_KEY) {
        if let Some(enabled) = enabled_value.as_bool() {
            tracing::debug!(enabled = enabled, "Found enable_watcher in settings");
            return Ok(enabled);
        }
    }

    // Default to false
    tracing::debug!("Using default enable_watcher: false");
    Ok(false)
}

/// Set whether the watcher is enabled
pub async fn set_enable_watcher(app: &AppHandle, enabled: bool) -> Result<(), String> {
    tracing::info!(enabled = enabled, "Setting enable_watcher in settings");

    let store = get_store(app).await?;
    store.set(ENABLE_WATCHER_KEY.to_string(), serde_json::json!(enabled));
    store.save().map_err(|e| format!("Failed to save settings: {}", e))?;

    Ok(())
}

// Tauri commands

/// Get all watch paths (Tauri command)
#[tauri::command]
pub async fn get_watch_paths_cmd(app: AppHandle) -> Result<Vec<String>, String> {
    let paths = get_watch_paths(&app).await?;
    Ok(paths.iter().map(|p| p.to_string_lossy().to_string()).collect())
}

/// Set all watch paths (Tauri command)
#[tauri::command]
pub async fn set_watch_paths_cmd(app: AppHandle, paths: Vec<String>) -> Result<(), String> {
    let path_bufs: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
    set_watch_paths(&app, path_bufs).await
}

/// Add a watch path (Tauri command)
#[tauri::command]
pub async fn add_watch_path_cmd(app: AppHandle, path: String) -> Result<(), String> {
    let mut paths = get_watch_paths(&app).await?;
    let new_path = PathBuf::from(path);

    // Avoid duplicates
    if !paths.contains(&new_path) {
        paths.push(new_path);
        set_watch_paths(&app, paths).await?;
    }

    Ok(())
}

/// Remove a watch path (Tauri command)
#[tauri::command]
pub async fn remove_watch_path_cmd(app: AppHandle, path: String) -> Result<(), String> {
    let mut paths = get_watch_paths(&app).await?;
    let path_to_remove = PathBuf::from(path);

    paths.retain(|p| p != &path_to_remove);
    set_watch_paths(&app, paths).await?;

    Ok(())
}

/// Get whether the watcher is enabled (Tauri command)
#[tauri::command]
pub async fn get_enable_watcher_cmd(app: AppHandle) -> Result<bool, String> {
    get_enable_watcher(&app).await
}

/// Set whether the watcher is enabled (Tauri command)
#[tauri::command]
pub async fn set_enable_watcher_cmd(app: AppHandle, enabled: bool) -> Result<(), String> {
    set_enable_watcher(&app, enabled).await
}
