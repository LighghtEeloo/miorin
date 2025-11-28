use miorin_core::prelude::*;
use serde_json;
use tauri::AppHandle;
use chrono::{DateTime, Utc};
use sqlx::sqlite::{SqlitePool, SqliteConnectOptions};
use sqlx::Row;
use tracing;

/// Get the database connection pool
async fn get_db_pool(app: &AppHandle) -> Result<SqlitePool, String> {
    use tauri::Manager;
    tracing::debug!("Getting app data directory");
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| {
            tracing::error!("Failed to get app data dir: {}", e);
            format!("Failed to get app data dir: {}", e)
        })?;
    
    tracing::debug!(app_data_dir = ?app_dir, "App data directory");
    
    tracing::debug!("Creating directory if it doesn't exist");
    std::fs::create_dir_all(&app_dir)
        .map_err(|e| {
            tracing::error!("Failed to create app data dir: {}", e);
            format!("Failed to create app data dir: {}", e)
        })?;
    
    let db_path = app_dir.join("miorin.db");
    tracing::debug!(db_path = ?db_path, "Database path");
    
    // Verify the directory exists and is writable
    if let Some(parent) = db_path.parent() {
        tracing::debug!(parent_dir = ?parent, "Verifying parent directory exists");
        if !parent.exists() {
            tracing::info!("Parent directory doesn't exist, creating");
            std::fs::create_dir_all(parent)
                .map_err(|e| {
                    tracing::error!("Failed to create parent directory: {}", e);
                    format!("Failed to create parent directory: {}", e)
                })?;
        }
        // Check if we can write to the directory
        let test_file = parent.join(".test_write");
        if let Err(e) = std::fs::write(&test_file, "test") {
            tracing::error!("Cannot write to directory: {}", e);
            return Err(format!("Cannot write to database directory: {}", e));
        }
        let _ = std::fs::remove_file(&test_file);
    }
    
    // Use SqliteConnectOptions which handles file paths better than connection strings
    // This avoids issues with spaces and special characters in paths
    let db_exists = db_path.exists();
    tracing::debug!(db_exists, "Database file exists");
    tracing::debug!("Using SqliteConnectOptions for better path handling");
    
    let options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(true);
    
    tracing::debug!("Connecting to database");
    let pool = SqlitePool::connect_with(options)
        .await
        .map_err(|e| {
            tracing::error!("Connection error: {}", e);
            format!("Failed to connect to database: {}", e)
        })?;
    tracing::info!("Successfully connected to database");
    
    // Run migrations if database is new or migrations haven't been applied
    if !db_exists {
        tracing::info!("Database is new, running migrations");
        run_migrations(&pool).await?;
    } else {
        // Check if migrations need to be run
        tracing::debug!("Checking if migrations are needed");
        if needs_migration(&pool).await? {
            tracing::info!("Running migrations");
            run_migrations(&pool).await?;
        }
    }
    
    Ok(pool)
}

/// Check if migrations need to be run
async fn needs_migration(pool: &SqlitePool) -> Result<bool, String> {
    // Check if the migrations table exists
    let result = sqlx::query(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='raw_entries'"
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Failed to check migrations: {}", e))?;
    
    Ok(result.is_none())
}

/// Run database migrations
async fn run_migrations(pool: &SqlitePool) -> Result<(), String> {
    tracing::info!("Starting migrations");
    
    // Read the migration SQL file
    let migration_sql = include_str!("../migrations/001_initial_schema.sql");
    
    // Execute the migration
    sqlx::raw_sql(migration_sql)
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Migration error: {}", e);
            format!("Failed to run migrations: {}", e)
        })?;
    
    tracing::info!("Migrations completed successfully");
    Ok(())
}

/// Get all raw entries
#[tauri::command]
pub async fn get_all_raw_entries(
    app: AppHandle,
) -> Result<Vec<Raw>, String> {
    let pool = get_db_pool(&app).await?;

    tracing::debug!("Executing query to get all raw entries");
    let rows = sqlx::query(
        "SELECT id, created_at, updated_at, vibe_json, source_json, content_json FROM raw_entries ORDER BY created_at DESC"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Query error: {}", e);
        format!("Query error: {}", e)
    })?;
    tracing::debug!(row_count = rows.len(), "Got rows from database");

    let mut entries = Vec::new();
    for row in rows {
        let id_str: String = row.get("id");
        let id = RawId(
            uuid::Uuid::parse_str(&id_str)
                .map_err(|e| format!("Invalid UUID: {}", e))?,
        );

        let created_at_str: String = row.get("created_at");
        let created_at: DateTime<Utc> = serde_json::from_str(&created_at_str)
            .map_err(|e| format!("Failed to parse created_at: {}", e))?;

        let updated_at_str: String = row.get("updated_at");
        let updated_at: DateTime<Utc> = serde_json::from_str(&updated_at_str)
            .map_err(|e| format!("Failed to parse updated_at: {}", e))?;

        let vibe: Option<Vibe> = if let Some(vibe_json) = row.try_get::<Option<String>, _>("vibe_json").ok().flatten() {
            if vibe_json == "null" || vibe_json.is_empty() {
                None
            } else {
                Some(serde_json::from_str(&vibe_json).map_err(|e| {
                    format!("Failed to parse vibe: {}", e)
                })?)
            }
        } else {
            None
        };

        let source_json: String = row.get("source_json");
        let source: RawSource = serde_json::from_str(&source_json)
            .map_err(|e| format!("Failed to parse source: {}", e))?;

        let content_json: String = row.get("content_json");
        let content: RawContent = serde_json::from_str(&content_json)
            .map_err(|e| format!("Failed to parse content: {}", e))?;

        entries.push(Raw {
            id,
            created_at,
            updated_at,
            vibe,
            inner: RawInner { source, content },
        });
    }

    tracing::debug!(entry_count = entries.len(), "Returning raw entries");
    Ok(entries)
}

/// Create a new raw entry
#[tauri::command]
pub async fn create_raw_entry(
    app: AppHandle,
    inner: RawInner,
) -> Result<Raw, String> {
    tracing::debug!("Creating new raw entry");
    let pool = get_db_pool(&app).await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            format!("Database error: {}", e)
        })?;
    tracing::debug!("Got database pool");

    let id = RawId(uuid::Uuid::now_v7());
    let now = Utc::now();

    let raw = Raw {
        id,
        created_at: now,
        updated_at: now,
        vibe: None,
        inner,
    };

    let id_str = id.0.to_string();
    let created_at_str = serde_json::to_string(&raw.created_at)
        .map_err(|e| format!("Failed to serialize created_at: {}", e))?;
    let updated_at_str = serde_json::to_string(&raw.updated_at)
        .map_err(|e| format!("Failed to serialize updated_at: {}", e))?;
    let source_json = serde_json::to_string(&raw.inner.source)
        .map_err(|e| format!("Failed to serialize source: {}", e))?;
    let content_json = serde_json::to_string(&raw.inner.content)
        .map_err(|e| format!("Failed to serialize content: {}", e))?;

    tracing::debug!("Executing INSERT query");
    sqlx::query(
        "INSERT INTO raw_entries (id, created_at, updated_at, vibe_json, source_json, content_json) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&id_str)
    .bind(&created_at_str)
    .bind(&updated_at_str)
    .bind("null")
    .bind(&source_json)
    .bind(&content_json)
    .execute(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Insert error: {}", e);
        format!("Failed to insert raw entry: {}", e)
    })?;
    tracing::info!(raw_entry_id = %id_str, "Successfully created raw entry");
    Ok(raw)
}

/// Update a raw entry
#[tauri::command]
pub async fn update_raw_entry(
    app: AppHandle,
    id: String,
    inner: Option<RawInner>,
    vibe: Option<Vibe>,
) -> Result<Raw, String> {
    let pool = get_db_pool(&app).await?;

    let raw_id = RawId(
        uuid::Uuid::parse_str(&id)
            .map_err(|e| format!("Invalid UUID: {}", e))?,
    );

    // Get existing entry
    let row = sqlx::query(
        "SELECT id, created_at, updated_at, vibe_json, source_json, content_json FROM raw_entries WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| format!("Query error: {}", e))?
    .ok_or("Raw entry not found")?;

    let created_at_str: String = row.get("created_at");
    let created_at: DateTime<Utc> = serde_json::from_str(&created_at_str)
        .map_err(|e| format!("Failed to parse created_at: {}", e))?;

    let source: RawSource = if let Some(ref new_inner) = inner {
        new_inner.source.clone()
    } else {
        let source_json: String = row.get("source_json");
        serde_json::from_str(&source_json)
            .map_err(|e| format!("Failed to parse source: {}", e))?
    };

    let content: RawContent = if let Some(ref new_inner) = inner {
        new_inner.content.clone()
    } else {
        let content_json: String = row.get("content_json");
        serde_json::from_str(&content_json)
            .map_err(|e| format!("Failed to parse content: {}", e))?
    };

    let new_vibe = if let Some(ref v) = vibe {
        Some(v.clone())
    } else {
        if let Some(vibe_json) = row.try_get::<Option<String>, _>("vibe_json").ok().flatten() {
            if vibe_json == "null" || vibe_json.is_empty() {
                None
            } else {
                Some(serde_json::from_str(&vibe_json).map_err(|e| {
                    format!("Failed to parse vibe: {}", e)
                })?)
            }
        } else {
            None
        }
    };

    let updated_at = Utc::now();

    let raw = Raw {
        id: raw_id,
        created_at,
        updated_at,
        vibe: new_vibe,
        inner: RawInner { source, content },
    };

    let updated_at_str = serde_json::to_string(&raw.updated_at)
        .map_err(|e| format!("Failed to serialize updated_at: {}", e))?;
    let vibe_json = if let Some(ref v) = raw.vibe {
        serde_json::to_string(v)
            .map_err(|e| format!("Failed to serialize vibe: {}", e))?
    } else {
        "null".to_string()
    };
    let source_json = serde_json::to_string(&raw.inner.source)
        .map_err(|e| format!("Failed to serialize source: {}", e))?;
    let content_json = serde_json::to_string(&raw.inner.content)
        .map_err(|e| format!("Failed to serialize content: {}", e))?;

    sqlx::query(
        "UPDATE raw_entries SET updated_at = ?, vibe_json = ?, source_json = ?, content_json = ? WHERE id = ?"
    )
    .bind(&updated_at_str)
    .bind(&vibe_json)
    .bind(&source_json)
    .bind(&content_json)
    .bind(&id)
    .execute(&pool)
    .await
    .map_err(|e| format!("Failed to update raw entry: {}", e))?;

    Ok(raw)
}

/// Delete a raw entry
#[tauri::command]
pub async fn delete_raw_entry(
    app: AppHandle,
    id: String,
) -> Result<(), String> {
    let pool = get_db_pool(&app).await?;

    sqlx::query("DELETE FROM raw_entries WHERE id = ?")
        .bind(&id)
        .execute(&pool)
        .await
        .map_err(|e| format!("Failed to delete raw entry: {}", e))?;

    Ok(())
}

/// Get all cubes
#[tauri::command]
pub async fn get_all_cubes(
    app: AppHandle,
) -> Result<Vec<Cube>, String> {
    let pool = get_db_pool(&app).await?;

    let rows = sqlx::query(
        "SELECT id, pin, content_json, created_at, updated_at, vibe_json FROM cubes ORDER BY created_at DESC"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Query error: {}", e))?;

    let mut cubes = Vec::new();
    for row in rows {
        let id_str: String = row.get("id");
        let id = CubeId(
            uuid::Uuid::parse_str(&id_str)
                .map_err(|e| format!("Invalid UUID: {}", e))?,
        );

        let pin: i64 = row.get("pin");
        let pin_bool = pin != 0;

        let content_json: String = row.get("content_json");
        let content: CubeContent = serde_json::from_str(&content_json)
            .map_err(|e| format!("Failed to parse content: {}", e))?;

        cubes.push(Cube { id, pin: pin_bool, content });
    }

    Ok(cubes)
}

/// Create a new cube
#[tauri::command]
pub async fn create_cube(
    app: AppHandle,
    pin: bool,
    content: CubeContent,
) -> Result<Cube, String> {
    let pool = get_db_pool(&app).await?;

    let id = CubeId(uuid::Uuid::now_v7());
    let now = Utc::now();

    let cube = Cube { id, pin, content };

    let id_str = id.0.to_string();
    let created_at_str = serde_json::to_string(&now)
        .map_err(|e| format!("Failed to serialize created_at: {}", e))?;
    let updated_at_str = serde_json::to_string(&now)
        .map_err(|e| format!("Failed to serialize updated_at: {}", e))?;
    let pin_int = if pin { 1 } else { 0 };
    let content_json = serde_json::to_string(&cube.content)
        .map_err(|e| format!("Failed to serialize content: {}", e))?;

    sqlx::query(
        "INSERT INTO cubes (id, pin, content_json, created_at, updated_at, vibe_json) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&id_str)
    .bind(pin_int)
    .bind(&content_json)
    .bind(&created_at_str)
    .bind(&updated_at_str)
    .bind("null")
    .execute(&pool)
    .await
    .map_err(|e| format!("Failed to insert cube: {}", e))?;

    Ok(cube)
}

/// Update a cube
#[tauri::command]
pub async fn update_cube(
    app: AppHandle,
    id: String,
    pin: Option<bool>,
    content: Option<CubeContent>,
) -> Result<Cube, String> {
    let pool = get_db_pool(&app).await?;

    let cube_id = CubeId(
        uuid::Uuid::parse_str(&id)
            .map_err(|e| format!("Invalid UUID: {}", e))?,
    );

    // Get existing cube
    let row = sqlx::query(
        "SELECT id, pin, content_json FROM cubes WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| format!("Query error: {}", e))?
    .ok_or("Cube not found")?;

    let current_pin: i64 = row.get("pin");
    let current_pin_bool = current_pin != 0;

    let current_content_json: String = row.get("content_json");
    let current_content: CubeContent = serde_json::from_str(&current_content_json)
        .map_err(|e| format!("Failed to parse content: {}", e))?;

    let new_pin = pin.unwrap_or(current_pin_bool);
    let new_content = content.unwrap_or(current_content);

    let cube = Cube {
        id: cube_id,
        pin: new_pin,
        content: new_content,
    };

    let updated_at_str = serde_json::to_string(&Utc::now())
        .map_err(|e| format!("Failed to serialize updated_at: {}", e))?;
    let pin_int = if cube.pin { 1 } else { 0 };
    let content_json = serde_json::to_string(&cube.content)
        .map_err(|e| format!("Failed to serialize content: {}", e))?;

    sqlx::query(
        "UPDATE cubes SET pin = ?, content_json = ?, updated_at = ? WHERE id = ?"
    )
    .bind(pin_int)
    .bind(&content_json)
    .bind(&updated_at_str)
    .bind(&id)
    .execute(&pool)
    .await
    .map_err(|e| format!("Failed to update cube: {}", e))?;

    Ok(cube)
}

/// Delete a cube
#[tauri::command]
pub async fn delete_cube(
    app: AppHandle,
    id: String,
) -> Result<(), String> {
    let pool = get_db_pool(&app).await?;

    sqlx::query("DELETE FROM cubes WHERE id = ?")
        .bind(&id)
        .execute(&pool)
        .await
        .map_err(|e| format!("Failed to delete cube: {}", e))?;

    Ok(())
}
