use std::path::{Path, PathBuf};
use tauri::AppHandle;
use tracing;
use miorin_core::prelude::*;
use crate::db::create_raw_entry;
use crate::blob::{store_blob_from_file, generate_thumbnail};

const SETTINGS_STORE_NAME: &str = "settings.json";
const WATCH_PATHS_KEY: &str = "watch_paths";
const DEVTOOLS_VISIBLE_KEY: &str = "devtools_visible";

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

/// Get all stored data from the settings store (Tauri command)
#[tauri::command]
pub async fn get_all_store_data_cmd(app: AppHandle) -> Result<serde_json::Value, String> {
    let store = get_store(&app).await?;
    
    // Collect all known keys and their values
    let mut all_data = serde_json::Map::new();
    
    // Get watch paths if it exists
    if let Some(paths_value) = store.get(WATCH_PATHS_KEY) {
        all_data.insert(WATCH_PATHS_KEY.to_string(), paths_value.clone());
    }
    
    // Try to get all keys by reading the store file directly
    use tauri::Manager;
    let app_data_dir = app.path().app_data_dir().map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let store_file_path = app_data_dir.join(SETTINGS_STORE_NAME);
    
    if store_file_path.exists() {
        match std::fs::read_to_string(&store_file_path) {
            Ok(content) => {
                match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(json_data) => {
                        if let Some(obj) = json_data.as_object() {
                            for (key, value) in obj {
                                all_data.insert(key.clone(), value.clone());
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse store file as JSON: {}", e);
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to read store file: {}", e);
            }
        }
    }
    
    Ok(serde_json::Value::Object(all_data))
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

/// Get devtools visibility setting
pub async fn get_devtools_visible(app: &AppHandle) -> Result<bool, String> {
    let store = get_store(app).await?;
    
    if let Some(value) = store.get(DEVTOOLS_VISIBLE_KEY) {
        if let Some(bool_val) = value.as_bool() {
            return Ok(bool_val);
        }
    }
    
    // Default to false if not set
    Ok(false)
}

/// Set devtools visibility setting
pub async fn set_devtools_visible(app: &AppHandle, visible: bool) -> Result<(), String> {
    let store = get_store(app).await?;
    store.set(DEVTOOLS_VISIBLE_KEY.to_string(), serde_json::json!(visible));
    store.save().map_err(|e| format!("Failed to save settings: {}", e))?;
    Ok(())
}

/// Get devtools visibility setting (Tauri command)
#[tauri::command]
pub async fn get_devtools_visible_cmd(app: AppHandle) -> Result<bool, String> {
    get_devtools_visible(&app).await
}

/// Set devtools visibility setting (Tauri command)
#[tauri::command]
pub async fn set_devtools_visible_cmd(app: AppHandle, visible: bool) -> Result<(), String> {
    set_devtools_visible(&app, visible).await
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

/// Import all existing files from a watch path as raw entries
#[tauri::command]
pub async fn import_files_from_path_cmd(app: AppHandle, path: String) -> Result<u32, String> {
    // Validate and canonicalize the path
    let canonical_path = validate_and_canonicalize_path(&path)?;
    
    tracing::info!(path = %canonical_path.display(), "Importing files from path");
    
    // Read directory entries
    let entries = std::fs::read_dir(&canonical_path)
        .map_err(|e| format!("Failed to read directory '{}': {}", canonical_path.display(), e))?;
    
    let mut imported_count = 0;
    let mut errors = Vec::new();
    
    // Process each entry
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let file_path = entry.path();
        
        // Skip if it's a directory
        if file_path.is_dir() {
            continue;
        }
        
        // Skip if it's not a file
        if !file_path.is_file() {
            continue;
        }
        
        // Process the file
        match import_file(&app, &file_path).await {
            Ok(_) => {
                imported_count += 1;
                tracing::debug!(file = %file_path.display(), "Imported file");
            }
            Err(e) => {
                let error_msg = format!("Failed to import file '{}': {}", file_path.display(), e);
                tracing::warn!(%error_msg);
                errors.push(error_msg);
            }
        }
    }
    
    if !errors.is_empty() {
        tracing::warn!(
            imported_count = imported_count,
            error_count = errors.len(),
            "Some files failed to import"
        );
    }
    
    tracing::info!(
        path = %canonical_path.display(),
        imported_count = imported_count,
        "Finished importing files"
    );
    
    Ok(imported_count)
}

/// Import a single file as a raw entry
async fn import_file(app: &AppHandle, file_path: &Path) -> Result<(), String> {
    use crate::watcher::{is_image_file, get_image_info, infer_mime_type};
    
    // Determine file type and read content
    let content = if is_image_file(file_path) {
        // For images, create ImageRaw
        let (width, height, format) = get_image_info(file_path)?;
        let blob_id = BlobId(uuid::Uuid::now_v7());
        
        // Store the image blob
        store_blob_from_file(app, blob_id, file_path).await?;

        // Generate thumbnail (max 200px)
        let thumbnail_blob_id = match generate_thumbnail(app, blob_id, 200).await {
            Ok((thumb_id, _)) => {
                tracing::debug!(thumbnail_blob_id = %thumb_id.0, "Generated thumbnail");
                Some(thumb_id)
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to generate thumbnail, continuing without it");
                None
            }
        };

        // Compute dominant color (simplified - just sample center pixel)
        let dominant_color_rgb = match image::open(file_path) {
            Ok(img) => {
                let w = img.width();
                let h = img.height();
                if w > 0 && h > 0 {
                    let rgb_img = img.to_rgb8();
                    let pixel = rgb_img.get_pixel(w / 2, h / 2);
                    Some([pixel[0], pixel[1], pixel[2]])
                } else {
                    None
                }
            }
            Err(_) => None,
        };
        
        RawContent::Image(ImageRaw {
            blob_id,
            width,
            height,
            format: Some(format),
            thumbnail_blob_id,
            dominant_color_rgb,
        })
    } else {
        // For text files, read the content
        let file_content = std::fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        let mime_type = infer_mime_type(file_path);
        let language = None; // Could be inferred later
        
        RawContent::Text(TextRaw { content: file_content, mime_type, language })
    };
    
    // Create Raw entry
    let inner = RawInner {
        source: RawSource::FileWatcher { original_path: file_path.to_path_buf() },
        content,
    };
    
    // Save to database
    create_raw_entry(app.clone(), inner).await?;
    
    Ok(())
}
