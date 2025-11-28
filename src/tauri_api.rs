#![allow(unused)]

use miorin_core::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use serde_json;
use serde::{Serialize, Deserialize};
use js_sys::Promise;
use web_sys;

/// Helper function to invoke Tauri commands
async fn invoke_tauri<T: Serialize, R: for<'de> Deserialize<'de>>(
    cmd: &str, args: T,
) -> Result<R, String> {
    web_sys::console::log_1(&format!("invoke_tauri: Calling command '{}'", cmd).into());

    // Get the Tauri invoke function from window.__TAURI__.core.invoke
    let window = web_sys::window().ok_or("Window not available")?;
    let tauri = js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__"))
        .map_err(|_| "Tauri API not available - make sure you're running in Tauri environment")?;
    web_sys::console::log_1(&"invoke_tauri: Found __TAURI__ object".into());

    let core = js_sys::Reflect::get(&tauri, &JsValue::from_str("core"))
        .map_err(|_| "Tauri core API not available")?;
    web_sys::console::log_1(&"invoke_tauri: Found core object".into());

    let invoke_fn = js_sys::Reflect::get(&core, &JsValue::from_str("invoke"))
        .map_err(|_| "Tauri invoke function not available")?;
    web_sys::console::log_1(&"invoke_tauri: Found invoke function".into());

    let args_js = serde_wasm_bindgen::to_value(&args).map_err(|e| {
        let err_msg = format!("Failed to serialize args: {}", e);
        web_sys::console::error_1(&err_msg.clone().into());
        err_msg
    })?;

    web_sys::console::log_1(&"invoke_tauri: Serialized args, calling invoke...".into());

    // Call the invoke function: invoke(cmd, args)
    let cmd_js = JsValue::from_str(cmd);
    let promise =
        js_sys::Function::from(invoke_fn).call2(&core, &cmd_js, &args_js).map_err(|e| {
            let err_msg = format!("Failed to call invoke function: {:?}", e);
            web_sys::console::error_1(&err_msg.clone().into());
            err_msg
        })?;

    let promise = Promise::from(promise);
    web_sys::console::log_1(&"invoke_tauri: Got promise, awaiting...".into());
    let result = JsFuture::from(promise).await.map_err(|e| {
        let err_msg = format!("Invoke error: {:?}", e);
        web_sys::console::error_1(&err_msg.clone().into());
        err_msg
    })?;

    web_sys::console::log_1(&"invoke_tauri: Got result, checking for errors...".into());
    // Check if result is an error (Tauri returns errors as objects with an error field)
    if js_sys::Reflect::has(&result, &JsValue::from_str("error")).unwrap_or(false) {
        let error_val = js_sys::Reflect::get(&result, &JsValue::from_str("error"))
            .ok()
            .unwrap_or(JsValue::NULL);
        let error_msg = if error_val.is_string() {
            error_val.as_string().unwrap_or_else(|| "Unknown error".to_string())
        } else {
            format!("{:?}", error_val)
        };
        web_sys::console::error_1(
            &format!("invoke_tauri: Command returned error: {}", error_msg).into(),
        );
        return Err(error_msg);
    }

    web_sys::console::log_1(&"invoke_tauri: Deserializing result...".into());
    let deserialized = serde_wasm_bindgen::from_value(result).map_err(|e| {
        let err_msg = format!("Failed to deserialize result: {}", e);
        web_sys::console::error_1(&err_msg.clone().into());
        err_msg
    })?;
    web_sys::console::log_1(
        &format!("invoke_tauri: Successfully completed command '{}'", cmd).into(),
    );
    Ok(deserialized)
}

/// Get all raw entries from the database
pub async fn get_all_raw_entries() -> Result<Vec<Raw>, String> {
    web_sys::console::log_1(&"tauri_api::get_all_raw_entries: Starting invoke...".into());
    let result: Result<Vec<Raw>, String> =
        invoke_tauri("get_all_raw_entries", serde_json::json!({})).await;
    match &result {
        | Ok(entries) => {
            web_sys::console::log_1(
                &format!("tauri_api::get_all_raw_entries: Success, got {} entries", entries.len())
                    .into(),
            );
        }
        | Err(e) => {
            web_sys::console::error_1(
                &format!("tauri_api::get_all_raw_entries: Error - {}", e).into(),
            );
        }
    }
    result
}

/// Get all cubes from the database
pub async fn get_all_cubes() -> Result<Vec<Cube>, String> {
    invoke_tauri("get_all_cubes", serde_json::json!({})).await
}

/// Create a new raw entry
pub async fn create_raw_entry(inner: RawInner) -> Result<Raw, String> {
    web_sys::console::log_1(&"tauri_api::create_raw_entry: Starting invoke...".into());
    let result: Result<Raw, String> =
        invoke_tauri("create_raw_entry", serde_json::json!({ "inner": inner })).await;
    match &result {
        | Ok(raw) => {
            web_sys::console::log_1(
                &format!(
                    "tauri_api::create_raw_entry: Success, created entry with id: {}",
                    raw.id.0
                )
                .into(),
            );
        }
        | Err(e) => {
            web_sys::console::error_1(
                &format!("tauri_api::create_raw_entry: Error - {}", e).into(),
            );
        }
    }
    result
}

/// Create a new cube
pub async fn create_cube(pin: bool, content: CubeContent) -> Result<Cube, String> {
    invoke_tauri("create_cube", serde_json::json!({ "pin": pin, "content": content })).await
}

/// Update a raw entry
pub async fn update_raw_entry(
    id: String, inner: Option<RawInner>, vibe: Option<Vibe>,
) -> Result<Raw, String> {
    invoke_tauri("update_raw_entry", serde_json::json!({ "id": id, "inner": inner, "vibe": vibe }))
        .await
}

/// Update a cube
pub async fn update_cube(
    id: String, pin: Option<bool>, content: Option<CubeContent>,
) -> Result<Cube, String> {
    invoke_tauri("update_cube", serde_json::json!({ "id": id, "pin": pin, "content": content }))
        .await
}

/// Delete a raw entry
pub async fn delete_raw_entry(id: String) -> Result<(), String> {
    invoke_tauri("delete_raw_entry", serde_json::json!({ "id": id })).await
}

/// Delete all raw entries (debug function)
pub async fn delete_all_raw_entries() -> Result<u32, String> {
    invoke_tauri("delete_all_raw_entries", serde_json::json!({})).await
}

/// Delete a cube
pub async fn delete_cube(id: String) -> Result<(), String> {
    invoke_tauri("delete_cube", serde_json::json!({ "id": id })).await
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WatchPathConfig {
    pub path: String,
    pub enabled: bool,
}

/// Get all watch path configurations
pub async fn get_watch_paths() -> Result<Vec<WatchPathConfig>, String> {
    invoke_tauri("get_watch_paths_cmd", serde_json::json!({})).await
}

/// Set all watch path configurations
pub async fn set_watch_paths(configs: Vec<WatchPathConfig>) -> Result<(), String> {
    invoke_tauri("set_watch_paths_cmd", serde_json::json!({ "configs": configs })).await
}

/// Add a watch path (returns the canonicalized path)
pub async fn add_watch_path(path: String) -> Result<String, String> {
    invoke_tauri("add_watch_path_cmd", serde_json::json!({ "path": path })).await
}

/// Remove a watch path
pub async fn remove_watch_path(path: String) -> Result<(), String> {
    invoke_tauri("remove_watch_path_cmd", serde_json::json!({ "path": path })).await
}

/// Toggle enabled state for a watch path
pub async fn toggle_watch_path_enabled(path: String, enabled: bool) -> Result<(), String> {
    invoke_tauri(
        "toggle_watch_path_enabled_cmd",
        serde_json::json!({ "path": path, "enabled": enabled }),
    )
    .await
}

/// Import all existing files from a watch path as raw entries
pub async fn import_files_from_path(path: String) -> Result<u32, String> {
    invoke_tauri("import_files_from_path_cmd", serde_json::json!({ "path": path })).await
}

/// Store a blob from binary data
pub async fn store_blob(blob_id: String, data: Vec<u8>) -> Result<(), String> {
    invoke_tauri("store_blob_cmd", serde_json::json!({ "blob_id": blob_id, "data": data })).await
}

/// Store a blob from a file path
pub async fn store_blob_from_file(blob_id: String, source_path: String) -> Result<(), String> {
    invoke_tauri("store_blob_from_file_cmd", serde_json::json!({ "blob_id": blob_id, "source_path": source_path })).await
}

/// Get blob data
pub async fn get_blob(blob_id: String) -> Result<Vec<u8>, String> {
    invoke_tauri("get_blob_cmd", serde_json::json!({ "blob_id": blob_id })).await
}

/// Check if a blob exists
pub async fn blob_exists(blob_id: String) -> Result<bool, String> {
    invoke_tauri("blob_exists_cmd", serde_json::json!({ "blob_id": blob_id })).await
}

/// Delete a blob
pub async fn delete_blob(blob_id: String) -> Result<(), String> {
    invoke_tauri("delete_blob_cmd", serde_json::json!({ "blob_id": blob_id })).await
}

/// Generate a thumbnail for an image blob
pub async fn generate_thumbnail(source_blob_id: String, max_size: u32) -> Result<String, String> {
    invoke_tauri("generate_thumbnail_cmd", serde_json::json!({ "source_blob_id": source_blob_id, "max_size": max_size })).await
}

/// Get all stored local data from the settings store
pub async fn get_all_store_data() -> Result<serde_json::Value, String> {
    invoke_tauri("get_all_store_data_cmd", serde_json::json!({})).await
}
