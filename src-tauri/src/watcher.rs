use miorin_core::prelude::*;
use notify::{Watcher, RecursiveMode, Event, EventKind};
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use tracing;
use std::sync::Arc;
use tokio::sync::mpsc;
use crate::db::create_raw_entry;
use crate::blob::{store_blob_from_file, generate_thumbnail, store_blob_data};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Start watching the configured directories
pub async fn start_file_watcher(app: AppHandle) -> Result<(), String> {
    use crate::settings::get_watch_paths;

    // Get enabled watch paths from settings
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
            if let Err(e) = handle_file_event(&app, &file_path).await {
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
        let tx = tx.clone();
        let mut watcher =
            notify::recommended_watcher(move |result: Result<Event, notify::Error>| {
                match result {
                    | Ok(event) => {
                        if let EventKind::Create(_) | EventKind::Modify(_) = event.kind {
                            for path in event.paths {
                                // Only process files (not directories)
                                if path.is_file() {
                                    // Check if the file is directly in the watched directory (not in a subdirectory)
                                    if let Some(parent) = path.parent() {
                                        if parent == watch_path_clone {
                                            if let Err(e) = tx.blocking_send(path.clone()) {
                                                tracing::error!("Failed to send file event: {}", e);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    | Err(e) => {
                        tracing::error!("Watcher error: {}", e);
                    }
                }
            })
            .map_err(|e| format!("Failed to create watcher for {:?}: {}", watch_path, e))?;

        // Watch the directory (non-recursive - only the directory itself)
        watcher
            .watch(watch_path, RecursiveMode::NonRecursive)
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
    app_clone.manage(Arc::new(watchers));

    Ok(())
}

/// Handle a file event by creating a Raw entry
async fn handle_file_event(app: &AppHandle, file_path: &Path) -> Result<(), String> {
    tracing::info!(file_path = ?file_path, "Handling file event");

    // Get file metadata for timestamps first (before creating blob)
    let file_metadata =
        std::fs::metadata(file_path).map_err(|e| format!("Failed to get file metadata: {}", e))?;

    // Use file modification time, or creation time if modification time is not available
    let file_time = file_metadata
        .modified()
        .or_else(|_| file_metadata.created())
        .map_err(|e| format!("Failed to get file time: {}", e))?;

    let file_datetime = chrono::DateTime::<chrono::Utc>::from(file_time);

    // Check for duplicate before creating blob
    use crate::db::find_duplicate_raw_entry;
    use crate::db::get_db_pool;
    let pool = get_db_pool(app).await.map_err(|e| format!("Failed to get database pool: {}", e))?;

    let source = RawSource::FileWatcher { original_path: file_path.to_path_buf() };
    if let Some(existing) = find_duplicate_raw_entry(&pool, &source, &file_datetime).await? {
        tracing::info!(
            raw_entry_id = %existing.id.0,
            file_path = ?file_path,
            "Raw entry with same source path and created date already exists, skipping"
        );
        return Ok(());
    }

    // Determine file type and read content
    let content = if is_image_file(file_path) {
        // For images, create ImageRaw
        let (width, height, format) = get_image_info(file_path)?;
        let blob_id = BlobId(uuid::Uuid::now_v7());

        // Store the image blob
        store_blob_from_file(app, blob_id, file_path).await?;

        // Generate thumbnail (max 200px)
        // Skip thumbnail generation for SVG files (the image crate doesn't support SVG)
        let thumbnail_blob_id = if format == "svg" {
            None
        } else {
            match generate_thumbnail(app, blob_id, 200).await {
                | Ok((thumb_id, _)) => {
                    tracing::debug!(thumbnail_blob_id = %thumb_id.0, "Generated thumbnail");
                    Some(thumb_id)
                }
                | Err(e) => {
                    tracing::warn!(error = %e, "Failed to generate thumbnail, continuing without it");
                    None
                }
            }
        };

        // Compute dominant color (simplified - just sample center pixel)
        // Skip for SVG files (the image crate doesn't support SVG)
        let dominant_color_rgb = if format == "svg" {
            None
        } else {
            match image::open(file_path) {
                | Ok(img) => {
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
                | Err(_) => None,
            }
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

    let created_at_str = serde_json::to_string(&file_datetime)
        .map_err(|e| format!("Failed to serialize file timestamp: {}", e))?;

    // Create Raw entry
    let inner = RawInner {
        source: RawSource::FileWatcher { original_path: file_path.to_path_buf() },
        content,
    };

    // Save to database with file timestamps
    let raw = create_raw_entry(app.clone(), inner, Some(created_at_str.clone()), Some(created_at_str))
        .await?;

    tracing::info!(file_path = ?file_path, "Created raw entry from file");

    // Emit event to notify frontend that a new raw entry was created
    use tauri::{Manager, Emitter};
    // Get all windows and emit to them
    for window in app.webview_windows().values() {
        if let Err(e) = window.emit("raw-entry-created", &raw) {
            tracing::warn!("Failed to emit raw-entry-created event to window: {}", e);
        }
    }

    Ok(())
}

/// Check if a file is an image
pub fn is_image_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_lower = ext.to_string_lossy().to_lowercase();
        matches!(
            ext_lower.as_str(),
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "ico" | "svg"
        )
    } else {
        false
    }
}

/// Get image dimensions and format
pub fn get_image_info(path: &Path) -> Result<(u32, u32, String), String> {
    let format = path.extension().and_then(|ext| ext.to_str()).unwrap_or("unknown").to_lowercase();
    
    // Special handling for SVG files
    if format == "svg" {
        // Try to parse SVG to get dimensions from viewBox or width/height attributes
        let svg_content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read SVG file: {}", e))?;
        
        // Default dimensions if we can't parse them
        let mut width = 800u32;
        let mut height = 600u32;
        
        // Try to extract viewBox
        if let Some(viewbox_start) = svg_content.find("viewBox=") {
            let after_equals = viewbox_start + 8;
            // Find the opening quote
            if let Some(quote_start_offset) = svg_content[after_equals..].find(|c: char| c == '"' || c == '\'') {
                let value_start = after_equals + quote_start_offset + 1;
                // Find the closing quote
                if let Some(quote_end_offset) = svg_content[value_start..].find(|c: char| c == '"' || c == '\'') {
                    if quote_end_offset > 0 {
                        let viewbox_str = &svg_content[value_start..value_start + quote_end_offset];
                        let parts: Vec<&str> = viewbox_str.split_whitespace().collect();
                        if parts.len() >= 4 {
                            if let (Ok(w), Ok(h)) = (parts[2].parse::<f64>(), parts[3].parse::<f64>()) {
                                width = w as u32;
                                height = h as u32;
                            }
                        }
                    }
                }
            }
        } else {
            // Try to extract width and height attributes
            if let Some(width_start) = svg_content.find("width=") {
                let after_equals = width_start + 6;
                // Find the opening quote
                if let Some(quote_start_offset) = svg_content[after_equals..].find(|c: char| c == '"' || c == '\'') {
                    let value_start = after_equals + quote_start_offset + 1;
                    // Find the closing quote or space
                    if let Some(quote_end_offset) = svg_content[value_start..].find(|c: char| c == '"' || c == '\'' || c == ' ' || c == '>') {
                        if quote_end_offset > 0 {
                            let width_str = &svg_content[value_start..value_start + quote_end_offset];
                            if let Ok(w) = width_str.parse::<f64>() {
                                width = w as u32;
                            }
                        }
                    }
                }
            }
            if let Some(height_start) = svg_content.find("height=") {
                let after_equals = height_start + 7;
                // Find the opening quote
                if let Some(quote_start_offset) = svg_content[after_equals..].find(|c: char| c == '"' || c == '\'') {
                    let value_start = after_equals + quote_start_offset + 1;
                    // Find the closing quote or space
                    if let Some(quote_end_offset) = svg_content[value_start..].find(|c: char| c == '"' || c == '\'' || c == ' ' || c == '>') {
                        if quote_end_offset > 0 {
                            let height_str = &svg_content[value_start..value_start + quote_end_offset];
                            if let Ok(h) = height_str.parse::<f64>() {
                                height = h as u32;
                            }
                        }
                    }
                }
            }
        }
        
        return Ok((width, height, format));
    }
    
    // For raster images, use the image crate
    let img = image::open(path).map_err(|e| format!("Failed to open image: {}", e))?;
    let (width, height) = (img.width(), img.height());

    Ok((width, height, format))
}

/// Infer MIME type from file extension
pub fn infer_mime_type(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            let ext_lower = ext.to_lowercase();
            match ext_lower.as_str() {
                | "txt" => "text/plain",
                | "md" => "text/markdown",
                | "html" | "htm" => "text/html",
                | "json" => "application/json",
                | "xml" => "application/xml",
                | "csv" => "text/csv",
                | "js" => "application/javascript",
                | "ts" => "application/typescript",
                | "py" => "text/x-python",
                | "rs" => "text/x-rust",
                | "go" => "text/x-go",
                | "java" => "text/x-java",
                | "cpp" | "cc" | "cxx" => "text/x-c++",
                | "c" => "text/x-c",
                | _ => "text/plain",
            }
        })
        .map(|s| s.to_string())
}

/// Start monitoring clipboard for changes
pub async fn start_clipboard_watcher(app: AppHandle) -> Result<(), String> {
    tracing::info!("Starting clipboard watcher");

    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        let mut last_hash: Option<u64> = None;
        let mut clipboard = match arboard::Clipboard::new() {
            Ok(clip) => clip,
            Err(e) => {
                tracing::error!("Failed to initialize clipboard in spawn: {}", e);
                return;
            }
        };

        loop {
            // Poll clipboard every 500ms
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            // Calculate hash of current clipboard content
            let current_hash = match get_clipboard_hash(&mut clipboard).await {
                Ok(Some(hash)) => hash,
                Ok(None) => {
                    // Clipboard is empty or error reading, skip
                    continue;
                }
                Err(e) => {
                    tracing::debug!("Error reading clipboard: {}", e);
                    continue;
                }
            };

            // Check if clipboard content has changed
            if let Some(last) = last_hash {
                if current_hash == last {
                    // No change, continue
                    continue;
                }
            }

            // Clipboard content has changed, process it
            last_hash = Some(current_hash);
            
            if let Err(e) = handle_clipboard_event(&app_clone, &mut clipboard).await {
                tracing::error!(error = %e, "Failed to handle clipboard event");
            }
        }
    });

    Ok(())
}

/// Get a hash of the current clipboard content to detect changes
async fn get_clipboard_hash(clipboard: &mut arboard::Clipboard) -> Result<Option<u64>, String> {
    // Try to get text first
    match clipboard.get_text() {
        Ok(text) => {
            let mut hasher = DefaultHasher::new();
            text.hash(&mut hasher);
            return Ok(Some(hasher.finish()));
        }
        Err(arboard::Error::ContentNotAvailable) => {
            // Text not available, try image
        }
        Err(e) => {
            return Err(format!("Failed to get clipboard text: {}", e));
        }
    }

    // Try to get image
    match clipboard.get_image() {
        Ok(img) => {
            let mut hasher = DefaultHasher::new();
            img.bytes.hash(&mut hasher);
            img.width.hash(&mut hasher);
            img.height.hash(&mut hasher);
            return Ok(Some(hasher.finish()));
        }
        Err(arboard::Error::ContentNotAvailable) => {
            // Neither text nor image available
            return Ok(None);
        }
        Err(e) => {
            return Err(format!("Failed to get clipboard image: {}", e));
        }
    }
}

/// Handle a clipboard change event by creating a Raw entry
async fn handle_clipboard_event(
    app: &AppHandle,
    clipboard: &mut arboard::Clipboard,
) -> Result<(), String> {
    tracing::info!("Handling clipboard event");

    let now = chrono::Utc::now();
    let created_at_str = serde_json::to_string(&now)
        .map_err(|e| format!("Failed to serialize timestamp: {}", e))?;

    // Check for duplicate before processing
    use crate::db::find_duplicate_raw_entry;
    use crate::db::get_db_pool;
    let pool = get_db_pool(app).await.map_err(|e| format!("Failed to get database pool: {}", e))?;

    // Try to get text first
    let content = match clipboard.get_text() {
        Ok(text) => {
            // Check for duplicate text entry
            let source = RawSource::Clipboard { application: None };
            if let Some(existing) = find_duplicate_raw_entry(&pool, &source, &now).await? {
                // Check if content matches
                if let RawContent::Text(TextRaw { content: existing_content, .. }) = &existing.inner.content {
                    if existing_content == &text {
                        tracing::debug!("Clipboard text entry already exists, skipping");
                        return Ok(());
                    }
                }
            }

            // Create text content
            RawContent::Text(TextRaw {
                content: text,
                mime_type: Some("text/plain".to_string()),
                language: None,
            })
        }
        Err(arboard::Error::ContentNotAvailable) => {
            // Text not available, try image
            match clipboard.get_image() {
                Ok(img) => {
                    // Check for duplicate image entry
                    let source = RawSource::Clipboard { application: None };
                    if let Some(_existing) = find_duplicate_raw_entry(&pool, &source, &now).await? {
                        // For images, we'll create a new entry even if it's a duplicate
                        // since we can't easily compare image content
                        tracing::debug!("Clipboard image entry may be duplicate, but creating anyway");
                    }

                    // Store the image blob
                    let blob_id = BlobId(uuid::Uuid::now_v7());
                    
                    // arboard gives us RGBA bytes, convert to image::RgbaImage
                    let rgba_image = image::RgbaImage::from_raw(
                        img.width as u32,
                        img.height as u32,
                        img.bytes.to_vec(),
                    )
                    .ok_or_else(|| format!("Failed to create image from clipboard data: invalid dimensions or data"))?;

                    // Convert to DynamicImage and save as PNG
                    let dynamic_img = image::DynamicImage::ImageRgba8(rgba_image);
                    let mut png_data = Vec::new();
                    {
                        let mut cursor = std::io::Cursor::new(&mut png_data);
                        dynamic_img.write_to(&mut cursor, image::ImageFormat::Png)
                            .map_err(|e| format!("Failed to encode PNG: {}", e))?;
                    }

                    let image_data = png_data;
                    let format = "png";

                    store_blob_data(app, blob_id, image_data).await?;

                    // Generate thumbnail
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

                    // Compute dominant color (sample center pixel)
                    let dominant_color_rgb = if img.width > 0 && img.height > 0 {
                        let center_idx = ((img.height / 2) * img.width + (img.width / 2)) * 4;
                        if center_idx + 3 < img.bytes.len() {
                            Some([
                                img.bytes[center_idx],
                                img.bytes[center_idx + 1],
                                img.bytes[center_idx + 2],
                            ])
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    RawContent::Image(ImageRaw {
                        blob_id,
                        width: img.width as u32,
                        height: img.height as u32,
                        format: Some(format.to_string()),
                        thumbnail_blob_id,
                        dominant_color_rgb,
                    })
                }
                Err(arboard::Error::ContentNotAvailable) => {
                    tracing::debug!("Clipboard is empty, skipping");
                    return Ok(());
                }
                Err(e) => {
                    return Err(format!("Failed to get clipboard image: {}", e));
                }
            }
        }
        Err(e) => {
            return Err(format!("Failed to get clipboard text: {}", e));
        }
    };

    // Create Raw entry
    let inner = RawInner {
        source: RawSource::Clipboard { application: None },
        content,
    };

    // Save to database
    let raw = create_raw_entry(
        app.clone(),
        inner,
        Some(created_at_str.clone()),
        Some(created_at_str),
    )
    .await?;

    tracing::info!("Created raw entry from clipboard");

    // Emit event to notify frontend that a new raw entry was created
    use tauri::{Manager, Emitter};
    for window in app.webview_windows().values() {
        if let Err(e) = window.emit("raw-entry-created", &raw) {
            tracing::warn!("Failed to emit raw-entry-created event to window: {}", e);
        }
    }

    Ok(())
}
