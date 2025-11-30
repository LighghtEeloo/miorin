use miorin_core::prelude::*;
use serde_json;
use tauri::AppHandle;
use chrono::{DateTime, Utc};
use sqlx::sqlite::{SqlitePool, SqliteConnectOptions};
use sqlx::Row;
use tracing;

/// Get the database connection pool
pub(crate) async fn get_db_pool(app: &AppHandle) -> Result<SqlitePool, String> {
    use tauri::Manager;
    tracing::debug!("Getting app data directory");
    let app_dir = app.path().app_data_dir().map_err(|e| {
        tracing::error!("Failed to get app data dir: {}", e);
        format!("Failed to get app data dir: {}", e)
    })?;

    tracing::debug!(app_data_dir = ?app_dir, "App data directory");

    tracing::debug!("Creating directory if it doesn't exist");
    std::fs::create_dir_all(&app_dir).map_err(|e| {
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
            std::fs::create_dir_all(parent).map_err(|e| {
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

    let options = SqliteConnectOptions::new().filename(&db_path).create_if_missing(true);

    tracing::debug!("Connecting to database");
    let pool = SqlitePool::connect_with(options).await.map_err(|e| {
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
    let result =
        sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='raw_entries'")
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
    sqlx::raw_sql(migration_sql).execute(pool).await.map_err(|e| {
        tracing::error!("Migration error: {}", e);
        format!("Failed to run migrations: {}", e)
    })?;

    tracing::info!("Migrations completed successfully");
    Ok(())
}

/// Get all raw entries
#[tauri::command]
pub async fn get_all_raw_entries(app: AppHandle) -> Result<Vec<Raw>, String> {
    let pool = get_db_pool(&app).await?;

    tracing::debug!("Executing query to get all raw entries");
    let rows = sqlx::query(
        "SELECT id, tags_json, created_at, updated_at, vibe_json, source_json, content_json FROM raw_entries ORDER BY created_at DESC"
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
        let id = RawId(uuid::Uuid::parse_str(&id_str).map_err(|e| format!("Invalid UUID: {}", e))?);

        let tags_json: String = row.get("tags_json");
        let tags: Vec<String> =
            serde_json::from_str(&tags_json).map_err(|e| format!("Failed to parse tags: {}", e))?;

        let created_at_str: String = row.get("created_at");
        let created_at: DateTime<Utc> = serde_json::from_str(&created_at_str)
            .map_err(|e| format!("Failed to parse created_at: {}", e))?;

        let updated_at_str: String = row.get("updated_at");
        let updated_at: DateTime<Utc> = serde_json::from_str(&updated_at_str)
            .map_err(|e| format!("Failed to parse updated_at: {}", e))?;

        let vibe: Option<Vibe> =
            if let Some(vibe_json) = row.try_get::<Option<String>, _>("vibe_json").ok().flatten() {
                if vibe_json == "null" || vibe_json.is_empty() {
                    None
                } else {
                    Some(
                        serde_json::from_str(&vibe_json)
                            .map_err(|e| format!("Failed to parse vibe: {}", e))?,
                    )
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
            tags,
            created_at,
            updated_at,
            vibe,
            inner: RawInner { source, content },
        });
    }

    tracing::debug!(entry_count = entries.len(), "Returning raw entries");
    Ok(entries)
}

/// Find existing raw entry by source path and created date
pub(crate) async fn find_duplicate_raw_entry(
    pool: &SqlitePool, source: &RawSource, created_at: &DateTime<Utc>,
) -> Result<Option<Raw>, String> {
    // Only check for FileWatcher sources with paths
    let source_path = if let RawSource::FileWatcher { original_path } = source {
        original_path
    } else {
        // For non-file sources, no deduplication
        return Ok(None);
    };

    // Get all raw entries and check for matching source paths and created dates
    let rows = sqlx::query(
        "SELECT id, tags_json, created_at, updated_at, vibe_json, source_json, content_json FROM raw_entries",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Query error: {}", e))?;

    let created_at_str = serde_json::to_string(created_at)
        .map_err(|e| format!("Failed to serialize created_at: {}", e))?;

    for row in rows {
        let row_created_at_str: String = row.get("created_at");

        // Check if created_at matches
        if row_created_at_str != created_at_str {
            continue;
        }

        let source_json: String = row.get("source_json");
        let existing_source: RawSource = serde_json::from_str(&source_json)
            .map_err(|e| format!("Failed to parse source: {}", e))?;

        // Check if this is a FileWatcher with the same path
        if let RawSource::FileWatcher { original_path: existing_path } = &existing_source {
            // Compare paths (normalize them for comparison)
            let existing_normalized =
                existing_path.canonicalize().unwrap_or_else(|_| existing_path.clone());
            let new_normalized = source_path.canonicalize().unwrap_or_else(|_| source_path.clone());

            // Compare normalized paths
            if existing_normalized == new_normalized {
                // Found a duplicate, return the existing entry
                let id_str: String = row.get("id");
                let id = RawId(
                    uuid::Uuid::parse_str(&id_str).map_err(|e| format!("Invalid UUID: {}", e))?,
                );

                let tags_json: String = row.get("tags_json");
                let tags: Vec<String> = serde_json::from_str(&tags_json)
                    .map_err(|e| format!("Failed to parse tags: {}", e))?;

                let created_at: DateTime<Utc> = serde_json::from_str(&row_created_at_str)
                    .map_err(|e| format!("Failed to parse created_at: {}", e))?;

                let updated_at_str: String = row.get("updated_at");
                let updated_at: DateTime<Utc> = serde_json::from_str(&updated_at_str)
                    .map_err(|e| format!("Failed to parse updated_at: {}", e))?;

                let vibe: Option<Vibe> = if let Some(vibe_json) =
                    row.try_get::<Option<String>, _>("vibe_json").ok().flatten()
                {
                    if vibe_json == "null" || vibe_json.is_empty() {
                        None
                    } else {
                        Some(
                            serde_json::from_str(&vibe_json)
                                .map_err(|e| format!("Failed to parse vibe: {}", e))?,
                        )
                    }
                } else {
                    None
                };

                let content_json: String = row.get("content_json");
                let content: RawContent = serde_json::from_str(&content_json)
                    .map_err(|e| format!("Failed to parse content: {}", e))?;

                return Ok(Some(Raw {
                    id,
                    tags,
                    created_at,
                    updated_at,
                    vibe,
                    inner: RawInner { source: existing_source, content },
                }));
            }
        }
    }

    Ok(None)
}

/// Get the last raw entry from clipboard source
pub(crate) async fn get_last_clipboard_entry(pool: &SqlitePool) -> Result<Option<Raw>, String> {
    let source_json = serde_json::to_string(&RawSource::Clipboard { application: None })
        .map_err(|e| format!("Failed to serialize clipboard source: {}", e))?;

    let row = sqlx::query(
        "SELECT id, tags_json, created_at, updated_at, vibe_json, source_json, content_json 
         FROM raw_entries 
         WHERE source_json = ? 
         ORDER BY created_at DESC 
         LIMIT 1",
    )
    .bind(&source_json)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Query error: {}", e))?;

    if let Some(row) = row {
        let id_str: String = row.get("id");
        let id = RawId(uuid::Uuid::parse_str(&id_str).map_err(|e| format!("Invalid UUID: {}", e))?);

        let tags_json: String = row.get("tags_json");
        let tags: Vec<String> =
            serde_json::from_str(&tags_json).map_err(|e| format!("Failed to parse tags: {}", e))?;

        let created_at_str: String = row.get("created_at");
        let created_at: DateTime<Utc> = serde_json::from_str(&created_at_str)
            .map_err(|e| format!("Failed to parse created_at: {}", e))?;

        let updated_at_str: String = row.get("updated_at");
        let updated_at: DateTime<Utc> = serde_json::from_str(&updated_at_str)
            .map_err(|e| format!("Failed to parse updated_at: {}", e))?;

        let vibe: Option<Vibe> =
            if let Some(vibe_json) = row.try_get::<Option<String>, _>("vibe_json").ok().flatten() {
                if vibe_json == "null" || vibe_json.is_empty() {
                    None
                } else {
                    Some(
                        serde_json::from_str(&vibe_json)
                            .map_err(|e| format!("Failed to parse vibe: {}", e))?,
                    )
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

        return Ok(Some(Raw {
            id,
            tags,
            created_at,
            updated_at,
            vibe,
            inner: RawInner { source, content },
        }));
    }

    Ok(None)
}

/// Create a new raw entry
#[tauri::command]
pub async fn create_raw_entry(
    app: AppHandle, inner: RawInner, created_at: Option<String>, updated_at: Option<String>,
) -> Result<Raw, String> {
    tracing::debug!("Creating new raw entry");
    let pool = get_db_pool(&app).await.map_err(|e| {
        tracing::error!("Database error: {}", e);
        format!("Database error: {}", e)
    })?;
    tracing::debug!("Got database pool");

    // Use provided timestamps or fall back to current time
    let created_at = if let Some(created_at_str) = created_at {
        serde_json::from_str(&created_at_str)
            .map_err(|e| format!("Failed to parse created_at: {}", e))?
    } else {
        Utc::now()
    };

    let updated_at = if let Some(updated_at_str) = updated_at {
        serde_json::from_str(&updated_at_str)
            .map_err(|e| format!("Failed to parse updated_at: {}", e))?
    } else {
        created_at // Use created_at as default for updated_at
    };

    // Check for duplicate entry with same source path and created date
    if let Some(existing) = find_duplicate_raw_entry(&pool, &inner.source, &created_at).await? {
        tracing::info!(
            raw_entry_id = %existing.id.0,
            "Raw entry with same source path and created date already exists, returning existing entry"
        );
        return Ok(existing);
    }

    let id = RawId(uuid::Uuid::now_v7());

    let raw = Raw { id, tags: vec![], created_at, updated_at, vibe: None, inner };

    let id_str = id.0.to_string();
    let tags_json =
        serde_json::to_string(&raw.tags).map_err(|e| format!("Failed to serialize tags: {}", e))?;
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
        "INSERT INTO raw_entries (id, tags_json, created_at, updated_at, vibe_json, source_json, content_json) VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id_str)
    .bind(&tags_json)
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
    app: AppHandle, id: String, inner: Option<RawInner>, vibe: Option<Vibe>,
) -> Result<Raw, String> {
    let pool = get_db_pool(&app).await?;

    let raw_id = RawId(uuid::Uuid::parse_str(&id).map_err(|e| format!("Invalid UUID: {}", e))?);

    // Get existing entry
    let row = sqlx::query(
        "SELECT id, tags_json, created_at, updated_at, vibe_json, source_json, content_json FROM raw_entries WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| format!("Query error: {}", e))?
    .ok_or("Raw entry not found")?;

    let tags_json: String = row.get("tags_json");
    let tags: Vec<String> =
        serde_json::from_str(&tags_json).map_err(|e| format!("Failed to parse tags: {}", e))?;

    let created_at_str: String = row.get("created_at");
    let created_at: DateTime<Utc> = serde_json::from_str(&created_at_str)
        .map_err(|e| format!("Failed to parse created_at: {}", e))?;

    let source: RawSource = if let Some(ref new_inner) = inner {
        new_inner.source.clone()
    } else {
        let source_json: String = row.get("source_json");
        serde_json::from_str(&source_json).map_err(|e| format!("Failed to parse source: {}", e))?
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
                Some(
                    serde_json::from_str(&vibe_json)
                        .map_err(|e| format!("Failed to parse vibe: {}", e))?,
                )
            }
        } else {
            None
        }
    };

    let updated_at = Utc::now();

    let raw = Raw {
        id: raw_id,
        tags,
        created_at,
        updated_at,
        vibe: new_vibe,
        inner: RawInner { source, content },
    };

    let tags_json =
        serde_json::to_string(&raw.tags).map_err(|e| format!("Failed to serialize tags: {}", e))?;
    let updated_at_str = serde_json::to_string(&raw.updated_at)
        .map_err(|e| format!("Failed to serialize updated_at: {}", e))?;
    let vibe_json = if let Some(ref v) = raw.vibe {
        serde_json::to_string(v).map_err(|e| format!("Failed to serialize vibe: {}", e))?
    } else {
        "null".to_string()
    };
    let source_json = serde_json::to_string(&raw.inner.source)
        .map_err(|e| format!("Failed to serialize source: {}", e))?;
    let content_json = serde_json::to_string(&raw.inner.content)
        .map_err(|e| format!("Failed to serialize content: {}", e))?;

    sqlx::query(
        "UPDATE raw_entries SET tags_json = ?, updated_at = ?, vibe_json = ?, source_json = ?, content_json = ? WHERE id = ?"
    )
    .bind(&tags_json)
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
pub async fn delete_raw_entry(app: AppHandle, id: String) -> Result<(), String> {
    let pool = get_db_pool(&app).await?;

    sqlx::query("DELETE FROM raw_entries WHERE id = ?")
        .bind(&id)
        .execute(&pool)
        .await
        .map_err(|e| format!("Failed to delete raw entry: {}", e))?;

    Ok(())
}

/// Delete all raw entries (debug function)
#[tauri::command]
pub async fn delete_all_raw_entries(app: AppHandle) -> Result<u32, String> {
    let pool = get_db_pool(&app).await?;

    let result = sqlx::query("DELETE FROM raw_entries")
        .execute(&pool)
        .await
        .map_err(|e| format!("Failed to delete all raw entries: {}", e))?;

    tracing::info!(deleted_count = result.rows_affected(), "Deleted all raw entries");
    Ok(result.rows_affected() as u32)
}

/// Get all cubes
#[tauri::command]
pub async fn get_all_cubes(app: AppHandle) -> Result<Vec<Cube>, String> {
    let pool = get_db_pool(&app).await?;

    let rows = sqlx::query(
        "SELECT id, tags_json, pin, content_json, created_at, updated_at, vibe_json FROM cubes ORDER BY created_at DESC"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Query error: {}", e))?;

    let mut cubes = Vec::new();
    for row in rows {
        let id_str: String = row.get("id");
        let id =
            CubeId(uuid::Uuid::parse_str(&id_str).map_err(|e| format!("Invalid UUID: {}", e))?);

        let tags_json: String = row.get("tags_json");
        let tags: Vec<String> =
            serde_json::from_str(&tags_json).map_err(|e| format!("Failed to parse tags: {}", e))?;

        let created_at_str: String = row.get("created_at");
        let created_at: DateTime<Utc> = serde_json::from_str(&created_at_str)
            .map_err(|e| format!("Failed to parse created_at: {}", e))?;

        let updated_at_str: String = row.get("updated_at");
        let updated_at: DateTime<Utc> = serde_json::from_str(&updated_at_str)
            .map_err(|e| format!("Failed to parse updated_at: {}", e))?;

        let vibe: Option<Vibe> =
            if let Some(vibe_json) = row.try_get::<Option<String>, _>("vibe_json").ok().flatten() {
                if vibe_json == "null" || vibe_json.is_empty() {
                    None
                } else {
                    Some(
                        serde_json::from_str(&vibe_json)
                            .map_err(|e| format!("Failed to parse vibe: {}", e))?,
                    )
                }
            } else {
                None
            };

        let pin: i64 = row.get("pin");
        let pin_bool = pin != 0;

        let content_json: String = row.get("content_json");
        let content: CubeContent = serde_json::from_str(&content_json)
            .map_err(|e| format!("Failed to parse content: {}", e))?;

        cubes.push(Cube {
            id,
            tags,
            created_at,
            updated_at,
            vibe,
            inner: CubeInner { pin: pin_bool, content },
        });
    }

    Ok(cubes)
}

/// Create a new cube
#[tauri::command]
pub async fn create_cube(app: AppHandle, pin: bool, content: CubeContent) -> Result<Cube, String> {
    let pool = get_db_pool(&app).await?;

    let id = CubeId(uuid::Uuid::now_v7());
    let now = Utc::now();

    let cube = Cube {
        id,
        tags: vec![],
        created_at: now,
        updated_at: now,
        vibe: None,
        inner: CubeInner { pin, content },
    };

    let id_str = id.0.to_string();
    let tags_json = serde_json::to_string(&cube.tags)
        .map_err(|e| format!("Failed to serialize tags: {}", e))?;
    let created_at_str = serde_json::to_string(&cube.created_at)
        .map_err(|e| format!("Failed to serialize created_at: {}", e))?;
    let updated_at_str = serde_json::to_string(&cube.updated_at)
        .map_err(|e| format!("Failed to serialize updated_at: {}", e))?;
    let pin_int = if cube.inner.pin { 1 } else { 0 };
    let content_json = serde_json::to_string(&cube.inner.content).map_err(|e| {
        tracing::error!("Failed to serialize content: {}", e);
        format!("Failed to serialize content: {}", e)
    })?;

    tracing::debug!(
        "Creating cube with pin={}, content_json length={}",
        pin_int,
        content_json.len()
    );

    sqlx::query(
        "INSERT INTO cubes (id, tags_json, pin, content_json, created_at, updated_at, vibe_json) VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id_str)
    .bind(&tags_json)
    .bind(pin_int)
    .bind(&content_json)
    .bind(&created_at_str)
    .bind(&updated_at_str)
    .bind("null")
    .execute(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to insert cube: {}", e);
        format!("Failed to insert cube: {}", e)
    })?;

    tracing::info!("Successfully created cube with id={}", id_str);
    Ok(cube)
}

/// Update a cube
#[tauri::command]
pub async fn update_cube(
    app: AppHandle, id: String, pin: Option<bool>, content: Option<CubeContent>,
) -> Result<Cube, String> {
    let pool = get_db_pool(&app).await?;

    let cube_id = CubeId(uuid::Uuid::parse_str(&id).map_err(|e| format!("Invalid UUID: {}", e))?);

    // Get existing cube
    let row = sqlx::query(
        "SELECT id, tags_json, pin, content_json, created_at, updated_at, vibe_json FROM cubes WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| format!("Query error: {}", e))?
    .ok_or("Cube not found")?;

    let tags_json: String = row.get("tags_json");
    let tags: Vec<String> =
        serde_json::from_str(&tags_json).map_err(|e| format!("Failed to parse tags: {}", e))?;

    let created_at_str: String = row.get("created_at");
    let created_at: DateTime<Utc> = serde_json::from_str(&created_at_str)
        .map_err(|e| format!("Failed to parse created_at: {}", e))?;

    let vibe: Option<Vibe> =
        if let Some(vibe_json) = row.try_get::<Option<String>, _>("vibe_json").ok().flatten() {
            if vibe_json == "null" || vibe_json.is_empty() {
                None
            } else {
                Some(
                    serde_json::from_str(&vibe_json)
                        .map_err(|e| format!("Failed to parse vibe: {}", e))?,
                )
            }
        } else {
            None
        };

    let current_pin: i64 = row.get("pin");
    let current_pin_bool = current_pin != 0;

    let current_content_json: String = row.get("content_json");
    let current_content: CubeContent = serde_json::from_str(&current_content_json)
        .map_err(|e| format!("Failed to parse content: {}", e))?;

    let new_pin = pin.unwrap_or(current_pin_bool);
    let new_content = content.unwrap_or(current_content);

    let updated_at = Utc::now();

    let cube = Cube {
        id: cube_id,
        tags,
        created_at,
        updated_at,
        vibe,
        inner: CubeInner { pin: new_pin, content: new_content },
    };

    let tags_json = serde_json::to_string(&cube.tags)
        .map_err(|e| format!("Failed to serialize tags: {}", e))?;
    let updated_at_str = serde_json::to_string(&cube.updated_at)
        .map_err(|e| format!("Failed to serialize updated_at: {}", e))?;
    let pin_int = if cube.inner.pin { 1 } else { 0 };
    let content_json = serde_json::to_string(&cube.inner.content)
        .map_err(|e| format!("Failed to serialize content: {}", e))?;

    sqlx::query(
        "UPDATE cubes SET tags_json = ?, pin = ?, content_json = ?, updated_at = ? WHERE id = ?",
    )
    .bind(&tags_json)
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
pub async fn delete_cube(app: AppHandle, id: String) -> Result<(), String> {
    let pool = get_db_pool(&app).await?;

    sqlx::query("DELETE FROM cubes WHERE id = ?")
        .bind(&id)
        .execute(&pool)
        .await
        .map_err(|e| format!("Failed to delete cube: {}", e))?;

    Ok(())
}
