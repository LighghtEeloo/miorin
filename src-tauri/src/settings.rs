use std::path::PathBuf;
use tauri::AppHandle;
use tracing;

const SETTINGS_STORE_NAME: &str = "settings.json";
const WATCH_PATHS_KEY: &str = "watch_paths";

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct WatchPathConfig {
    pub path: String,
    pub enabled: bool,
}

/// Get the store helper function
async fn get_store(
    app: &AppHandle,
) -> Result<std::sync::Arc<tauri_plugin_store::Store<tauri::Wry>>, String> {
    use tauri_plugin_store::StoreBuilder;
    let store_path = std::path::PathBuf::from(SETTINGS_STORE_NAME);
    StoreBuilder::new(app, store_path).build().map_err(|e| format!("Failed to build store: {}", e))
}

/// Get all watch path configurations from settings
pub async fn get_watch_path_configs(app: &AppHandle) -> Result<Vec<WatchPathConfig>, String> {
    let store = get_store(app).await?;

    if let Some(paths_value) = store.get(WATCH_PATHS_KEY) {
        if let Some(paths_array) = paths_value.as_array() {
            let configs: Vec<WatchPathConfig> =
                paths_array.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect();
            tracing::debug!(watch_paths = ?configs, "Found watch paths in settings");
            return Ok(configs);
        }
    }

    // Return empty vector if no paths are configured
    tracing::debug!("No watch paths configured");
    Ok(vec![])
}

/// Get all watch paths (for backward compatibility and watcher)
pub async fn get_watch_paths(app: &AppHandle) -> Result<Vec<PathBuf>, String> {
    let configs = get_watch_path_configs(app).await?;
    Ok(configs.iter().filter(|c| c.enabled).map(|c| PathBuf::from(&c.path)).collect())
}

/// Set watch path configurations in settings
pub async fn set_watch_path_configs(
    app: &AppHandle, configs: Vec<WatchPathConfig>,
) -> Result<(), String> {
    tracing::info!(watch_paths = ?configs, "Setting watch paths in settings");

    let store = get_store(app).await?;
    store.set(WATCH_PATHS_KEY.to_string(), serde_json::json!(configs));
    store.save().map_err(|e| format!("Failed to save settings: {}", e))?;

    Ok(())
}

// Tauri commands

/// Get all watch path configurations (Tauri command)
#[tauri::command]
pub async fn get_watch_paths_cmd(app: AppHandle) -> Result<Vec<WatchPathConfig>, String> {
    get_watch_path_configs(&app).await
}

/// Set all watch path configurations (Tauri command)
#[tauri::command]
pub async fn set_watch_paths_cmd(
    app: AppHandle, configs: Vec<WatchPathConfig>,
) -> Result<(), String> {
    set_watch_path_configs(&app, configs).await
}

/// Expand shell variables and tilde in a path string
fn expand_shell_path(path_str: &str) -> Result<String, String> {
    shellexpand::full(path_str)
        .map(|s| s.into_owned())
        .map_err(|e| format!("Failed to expand shell path '{}': {}", path_str, e))
}

/// Validate and canonicalize a path
/// Expands shell variables (~, $HOME, etc.), resolves relative paths, and validates the path exists and is a directory
pub fn validate_and_canonicalize_path(path_str: &str) -> Result<PathBuf, String> {
    // First, expand shell variables and tilde
    let expanded_str = expand_shell_path(path_str)?;
    let mut path = PathBuf::from(&expanded_str);

    // Resolve to absolute path if relative
    if path.is_relative() {
        let current_dir = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;
        path = current_dir.join(&path);
    }

    // Canonicalize (resolve symlinks, etc.)
    let canonical = path.canonicalize().map_err(|e| {
        // If path doesn't exist, provide a more helpful error
        if e.kind() == std::io::ErrorKind::NotFound {
            format!("Path does not exist: {}", path_str)
        } else {
            format!("Failed to resolve path '{}': {}", path_str, e)
        }
    })?;

    // Validate it's a directory
    if !canonical.is_dir() {
        return Err(format!("Path is not a directory: {}", canonical.display()));
    }

    // Check if directory is readable
    std::fs::read_dir(&canonical)
        .map_err(|e| format!("Cannot read directory '{}': {}", canonical.display(), e))?;

    Ok(canonical)
}

/// Add a watch path (Tauri command)
#[tauri::command]
pub async fn add_watch_path_cmd(app: AppHandle, path: String) -> Result<String, String> {
    // Validate and canonicalize the path
    let canonical_path = validate_and_canonicalize_path(&path)?;
    let canonical_str = canonical_path.to_string_lossy().to_string();

    let mut configs = get_watch_path_configs(&app).await?;

    // Check for duplicates using canonical paths
    let canonical_paths: Vec<PathBuf> = configs
        .iter()
        .map(|c| PathBuf::from(&c.path).canonicalize().unwrap_or_else(|_| PathBuf::from(&c.path)))
        .collect();

    let canonical_path_normalized =
        canonical_path.canonicalize().unwrap_or_else(|_| canonical_path.clone());

    if canonical_paths.contains(&canonical_path_normalized) {
        return Err(format!("Path already exists: {}", canonical_str));
    }

    // Add the canonical path with enabled=false by default
    configs.push(WatchPathConfig { path: canonical_str.clone(), enabled: false });
    set_watch_path_configs(&app, configs).await?;

    tracing::info!(original_path = %path, canonical_path = %canonical_str, "Added watch path");

    Ok(canonical_str)
}

/// Remove a watch path (Tauri command)
#[tauri::command]
pub async fn remove_watch_path_cmd(app: AppHandle, path: String) -> Result<(), String> {
    let mut configs = get_watch_path_configs(&app).await?;
    configs.retain(|c| c.path != path);
    set_watch_path_configs(&app, configs).await?;
    Ok(())
}

/// Toggle enabled state for a watch path (Tauri command)
#[tauri::command]
pub async fn toggle_watch_path_enabled_cmd(
    app: AppHandle, path: String, enabled: bool,
) -> Result<(), String> {
    let mut configs = get_watch_path_configs(&app).await?;
    for config in &mut configs {
        if config.path == path {
            config.enabled = enabled;
            break;
        }
    }
    set_watch_path_configs(&app, configs).await?;
    Ok(())
}
