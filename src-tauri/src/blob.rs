use miorin_core::prelude::*;
use tauri::AppHandle;
use std::path::{Path, PathBuf};
use tracing;

/// Get the blob storage directory path
fn get_blob_dir(app: &AppHandle) -> Result<PathBuf, String> {
    use tauri::Manager;
    let app_dir = app.path().app_data_dir().map_err(|e| {
        tracing::error!("Failed to get app data dir: {}", e);
        format!("Failed to get app data dir: {}", e)
    })?;

    let blob_dir = app_dir.join("blobs");
    
    // Create directory if it doesn't exist
    std::fs::create_dir_all(&blob_dir).map_err(|e| {
        tracing::error!("Failed to create blob directory: {}", e);
        format!("Failed to create blob directory: {}", e)
    })?;

    Ok(blob_dir)
}

/// Get the file path for a blob ID
fn get_blob_path(app: &AppHandle, blob_id: BlobId) -> Result<PathBuf, String> {
    let blob_dir = get_blob_dir(app)?;
    let filename = format!("{}", blob_id.0);
    Ok(blob_dir.join(filename))
}

/// Store a blob from binary data
pub async fn store_blob_data(app: &AppHandle, blob_id: BlobId, data: Vec<u8>) -> Result<(), String> {
    let blob_path = get_blob_path(app, blob_id)?;
    
    tracing::debug!(blob_id = %blob_id.0, path = ?blob_path, "Storing blob");
    
    tokio::fs::write(&blob_path, data).await.map_err(|e| {
        tracing::error!(blob_id = %blob_id.0, error = %e, "Failed to write blob");
        format!("Failed to write blob: {}", e)
    })?;
    
    tracing::info!(blob_id = %blob_id.0, "Successfully stored blob");
    Ok(())
}

/// Store a blob from a file path (copy the file)
pub async fn store_blob_from_file(
    app: &AppHandle, blob_id: BlobId, source_path: &Path,
) -> Result<(), String> {
    let blob_path = get_blob_path(app, blob_id)?;
    
    tracing::debug!(blob_id = %blob_id.0, source = ?source_path, path = ?blob_path, "Storing blob from file");
    
    tokio::fs::copy(source_path, &blob_path).await.map_err(|e| {
        tracing::error!(blob_id = %blob_id.0, error = %e, "Failed to copy blob file");
        format!("Failed to copy blob file: {}", e)
    })?;
    
    tracing::info!(blob_id = %blob_id.0, "Successfully stored blob from file");
    Ok(())
}

/// Get blob data
pub async fn get_blob_data(app: &AppHandle, blob_id: BlobId) -> Result<Vec<u8>, String> {
    let blob_path = get_blob_path(app, blob_id)?;
    
    if !blob_path.exists() {
        return Err(format!("Blob not found: {}", blob_id.0));
    }
    
    tokio::fs::read(&blob_path).await.map_err(|e| {
        tracing::error!(blob_id = %blob_id.0, error = %e, "Failed to read blob");
        format!("Failed to read blob: {}", e)
    })
}

/// Check if a blob exists
pub async fn blob_exists(app: &AppHandle, blob_id: BlobId) -> bool {
    match get_blob_path(app, blob_id) {
        Ok(path) => path.exists(),
        Err(_) => false,
    }
}

/// Delete a blob
pub async fn delete_blob(app: &AppHandle, blob_id: BlobId) -> Result<(), String> {
    let blob_path = get_blob_path(app, blob_id)?;
    
    if !blob_path.exists() {
        tracing::warn!(blob_id = %blob_id.0, "Blob does not exist, skipping deletion");
        return Ok(());
    }
    
    tokio::fs::remove_file(&blob_path).await.map_err(|e| {
        tracing::error!(blob_id = %blob_id.0, error = %e, "Failed to delete blob");
        format!("Failed to delete blob: {}", e)
    })?;
    
    tracing::info!(blob_id = %blob_id.0, "Successfully deleted blob");
    Ok(())
}

/// Generate a thumbnail for an image blob
/// Returns the thumbnail blob ID and the thumbnail data
pub async fn generate_thumbnail(
    app: &AppHandle, source_blob_id: BlobId, max_size: u32,
) -> Result<(BlobId, Vec<u8>), String> {
    // Read the source image
    let image_data = get_blob_data(app, source_blob_id).await?;
    
    // Decode the image
    let img = image::load_from_memory(&image_data)
        .map_err(|e| format!("Failed to decode image: {}", e))?;
    
    // Calculate thumbnail dimensions while maintaining aspect ratio
    let (width, height) = (img.width(), img.height());
    let (thumb_width, thumb_height) = if width > height {
        let ratio = max_size as f32 / width as f32;
        (max_size, (height as f32 * ratio) as u32)
    } else {
        let ratio = max_size as f32 / height as f32;
        ((width as f32 * ratio) as u32, max_size)
    };
    
    // Resize the image
    let thumbnail = img.thumbnail(thumb_width, thumb_height);
    
    // Convert to RGB8 if needed
    let rgb_thumbnail = thumbnail.to_rgb8();
    
    // Encode as JPEG (smaller file size for thumbnails)
    let mut thumbnail_data = Vec::new();
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut thumbnail_data, 85);
    encoder.encode(
        rgb_thumbnail.as_raw(),
        rgb_thumbnail.width(),
        rgb_thumbnail.height(),
        image::ExtendedColorType::Rgb8,
    )
    .map_err(|e| format!("Failed to encode thumbnail: {}", e))?;
    
    // Create a new blob ID for the thumbnail
    let thumbnail_blob_id = BlobId(uuid::Uuid::now_v7());
    
    // Store the thumbnail
    store_blob_data(app, thumbnail_blob_id, thumbnail_data.clone()).await?;
    
    tracing::info!(
        source_blob_id = %source_blob_id.0,
        thumbnail_blob_id = %thumbnail_blob_id.0,
        "Generated thumbnail"
    );
    
    Ok((thumbnail_blob_id, thumbnail_data))
}

// Tauri commands

/// Store a blob from binary data (Tauri command)
#[tauri::command]
pub async fn store_blob_cmd(
    app: AppHandle, blob_id: String, data: Vec<u8>,
) -> Result<(), String> {
    let blob_id = BlobId(
        uuid::Uuid::parse_str(&blob_id)
            .map_err(|e| format!("Invalid blob ID: {}", e))?,
    );
    store_blob_data(&app, blob_id, data).await
}

/// Store a blob from a file path (Tauri command)
#[tauri::command]
pub async fn store_blob_from_file_cmd(
    app: AppHandle, blob_id: String, source_path: String,
) -> Result<(), String> {
    let blob_id = BlobId(
        uuid::Uuid::parse_str(&blob_id)
            .map_err(|e| format!("Invalid blob ID: {}", e))?,
    );
    let source_path = PathBuf::from(source_path);
    store_blob_from_file(&app, blob_id, &source_path).await
}

/// Get blob data (Tauri command)
#[tauri::command]
pub async fn get_blob_cmd(app: AppHandle, blob_id: String) -> Result<Vec<u8>, String> {
    let blob_id = BlobId(
        uuid::Uuid::parse_str(&blob_id)
            .map_err(|e| format!("Invalid blob ID: {}", e))?,
    );
    get_blob_data(&app, blob_id).await
}

/// Check if a blob exists (Tauri command)
#[tauri::command]
pub async fn blob_exists_cmd(app: AppHandle, blob_id: String) -> Result<bool, String> {
    let blob_id = BlobId(
        uuid::Uuid::parse_str(&blob_id)
            .map_err(|e| format!("Invalid blob ID: {}", e))?,
    );
    Ok(blob_exists(&app, blob_id).await)
}

/// Delete a blob (Tauri command)
#[tauri::command]
pub async fn delete_blob_cmd(app: AppHandle, blob_id: String) -> Result<(), String> {
    let blob_id = BlobId(
        uuid::Uuid::parse_str(&blob_id)
            .map_err(|e| format!("Invalid blob ID: {}", e))?,
    );
    delete_blob(&app, blob_id).await
}

/// Generate a thumbnail for an image blob (Tauri command)
#[tauri::command]
pub async fn generate_thumbnail_cmd(
    app: AppHandle, source_blob_id: String, max_size: u32,
) -> Result<String, String> {
    let source_blob_id = BlobId(
        uuid::Uuid::parse_str(&source_blob_id)
            .map_err(|e| format!("Invalid blob ID: {}", e))?,
    );
    let (thumbnail_blob_id, _) = generate_thumbnail(&app, source_blob_id, max_size).await?;
    Ok(thumbnail_blob_id.0.to_string())
}

