use miorin_core::prelude::*;
use notify::{Watcher, RecursiveMode, Event, EventKind};
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use tracing;
use std::sync::Arc;
use tokio::sync::mpsc;
use crate::db::create_raw_entry;

/// Start watching the configured directories
pub async fn start_file_watcher(app: AppHandle) -> Result<(), String> {
    use crate::settings::{get_watch_paths, get_enable_watcher};
    
    // Check if watcher is enabled
    let enabled = get_enable_watcher(&app).await?;
    if !enabled {
        tracing::info!("File watcher is disabled in settings");
        return Ok(());
    }
    
    // Get watch paths from settings
    let watch_paths = get_watch_paths(&app).await?;
    
    if watch_paths.is_empty() {
        tracing::warn!("No watch paths configured");
        return Ok(());
    }
    
    tracing::info!(watch_paths = ?watch_paths, "Starting file watcher for multiple paths");
    
    // Create a channel for file events
    let (tx, mut rx) = mpsc::channel::<PathBuf>(100);
    
    // Spawn task to handle file events
    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        while let Some(file_path) = rx.recv().await {
            tracing::debug!(file_path = ?file_path, "Processing file event");
            if let Err(e) = handle_file_event(&app_clone, &file_path).await {
                tracing::error!(file_path = ?file_path, error = %e, "Failed to handle file event");
            }
        }
    });
    
    // Create watchers for each path
    let mut watchers = Vec::new();
    
    for watch_path in &watch_paths {
        // Check if directory exists
        if !watch_path.exists() {
            tracing::warn!(watch_path = ?watch_path, "Watched directory does not exist, skipping");
            continue;
        }
        
        if !watch_path.is_dir() {
            tracing::warn!(watch_path = ?watch_path, "Watch path is not a directory, skipping");
            continue;
        }
        
        // Create the watcher with async handler
        let watch_path_clone = watch_path.clone();
        let tx_clone = tx.clone();
        let mut watcher = notify::recommended_watcher(
            move |result: Result<Event, notify::Error>| {
                match result {
                    Ok(event) => {
                        if let EventKind::Create(_) | EventKind::Modify(_) = event.kind {
                            for path in event.paths {
                                // Only process files (not directories)
                                if path.is_file() {
                                    // Check if the file is directly in the watched directory (not in a subdirectory)
                                    if let Some(parent) = path.parent() {
                                        if parent == watch_path_clone {
                                            if let Err(e) = tx_clone.blocking_send(path.clone()) {
                                                tracing::error!("Failed to send file event: {}", e);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Watcher error: {}", e);
                    }
                }
            },
        )
        .map_err(|e| format!("Failed to create watcher for {:?}: {}", watch_path, e))?;
        
        // Watch the directory (non-recursive - only the directory itself)
        watcher.watch(watch_path, RecursiveMode::NonRecursive)
            .map_err(|e| format!("Failed to watch directory {:?}: {}", watch_path, e))?;
        
        tracing::info!(watch_path = ?watch_path, "File watcher started for path");
        watchers.push(watcher);
    }
    
    if watchers.is_empty() {
        tracing::warn!("No valid watch paths, file watcher not started");
        return Ok(());
    }
    
    tracing::info!(count = watchers.len(), "File watchers started successfully");
    
    // Keep the watchers alive by storing them in app state
    use tauri::Manager;
    app.manage(Arc::new(watchers));
    
    Ok(())
}

/// Handle a file event by creating a Raw entry
async fn handle_file_event(app: &AppHandle, file_path: &Path) -> Result<(), String> {
    tracing::info!(file_path = ?file_path, "Handling file event");
    
    // Determine file type and read content
    let content = if is_image_file(file_path) {
        // For images, create ImageRaw
        let (width, height, format) = get_image_info(file_path)?;
        let blob_id = BlobId(uuid::Uuid::now_v7());
        
        // TODO: Actually store the image blob and generate thumbnail
        // For now, just create the entry with metadata
        
        RawContent::Image(ImageRaw {
            blob_id,
            width,
            height,
            format: Some(format),
            thumbnail_blob_id: None,
            dominant_color_rgb: None,
        })
    } else {
        // For text files, read the content
        let file_content = std::fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        let mime_type = infer_mime_type(file_path);
        let language = None; // Could be inferred later
        
        RawContent::Text(TextRaw {
            content: file_content,
            mime_type,
            language,
        })
    };
    
    // Create Raw entry
    let inner = RawInner {
        source: RawSource::FileWatcher {
            original_path: file_path.to_path_buf(),
        },
        content,
    };
    
    // Save to database
    create_raw_entry(app.clone(), inner).await?;
    
    tracing::info!(file_path = ?file_path, "Created raw entry from file");
    Ok(())
}

/// Check if a file is an image
fn is_image_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_lower = ext.to_string_lossy().to_lowercase();
        matches!(ext_lower.as_str(), "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "ico" | "svg")
    } else {
        false
    }
}

/// Get image dimensions and format
fn get_image_info(path: &Path) -> Result<(u32, u32, String), String> {
    let img = image::open(path)
        .map_err(|e| format!("Failed to open image: {}", e))?;
    
    let (width, height) = (img.width(), img.height());
    let format = path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("unknown")
        .to_lowercase();
    
    Ok((width, height, format))
}

/// Infer MIME type from file extension
fn infer_mime_type(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            let ext_lower = ext.to_lowercase();
            match ext_lower.as_str() {
                "txt" => "text/plain",
                "md" => "text/markdown",
                "html" | "htm" => "text/html",
                "json" => "application/json",
                "xml" => "application/xml",
                "csv" => "text/csv",
                "js" => "application/javascript",
                "ts" => "application/typescript",
                "py" => "text/x-python",
                "rs" => "text/x-rust",
                "go" => "text/x-go",
                "java" => "text/x-java",
                "cpp" | "cc" | "cxx" => "text/x-c++",
                "c" => "text/x-c",
                _ => "text/plain",
            }
        })
        .map(|s| s.to_string())
}

