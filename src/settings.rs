use crate::tauri_api;
use leptos::prelude::*;
use leptos_icons::Icon;
use icondata::{LuDownload, LuEye, LuEyeOff, LuFolderOpen, LuPlus, LuRefreshCw, LuTrash2, LuX};
use styled::style;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen::JsCast;


#[component]
fn WatchPathItem(
    config: tauri_api::WatchPathConfig,
    on_toggle: impl Fn(String, bool) + 'static,
    on_remove: impl Fn(String) + 'static,
    on_import: impl Fn(String) + 'static,
) -> impl IntoView {
    let path_item_styles = style! {
        .path-item {
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 10px;
            margin-bottom: 8px;
            background-color: var(--color-bg-secondary, #f8f9fa);
            border-radius: 4px;
            border: 1px solid var(--color-border, #dee2e6);
        }
        .path-text {
            flex: 1;
            margin-right: 10px;
            word-break: break-all;
            color: var(--color-text-secondary, #6c757d);
            font-family: monospace;
            font-size: 13px;
        }
        .button {
            padding: 8px 16px;
            border: none;
            border-radius: 4px;
            cursor: pointer;
            font-size: 14px;
            font-weight: 500;
            transition: background-color 0.2s;
            display: flex;
            align-items: center;
            justify-content: center;
            gap: 6px;
        }
        .button-danger {
            background-color: var(--color-danger, #dc3545);
            color: white;
            padding: 8px;
        }
        .button-danger:hover {
            background-color: var(--color-danger-hover, #c82333);
        }
        .button-import {
            background-color: var(--color-primary, #007bff);
            color: white;
            padding: 8px;
        }
        .button-import:hover {
            background-color: var(--color-primary-hover, #0056b3);
        }
        .toggle-group {
            display: flex;
            align-items: center;
            gap: 8px;
        }
        .toggle-switch {
            position: relative;
            display: inline-block;
            width: 44px;
            height: 24px;
        }
        .toggle-switch input {
            opacity: 0;
            width: 0;
            height: 0;
        }
        .toggle-slider {
            position: absolute;
            cursor: pointer;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background-color: var(--color-toggle-bg, #ccc);
            transition: 0.3s;
            border-radius: 24px;
        }
        .toggle-slider:before {
            position: absolute;
            content: "";
            height: 18px;
            width: 18px;
            left: 3px;
            bottom: 3px;
            background-color: white;
            transition: 0.3s;
            border-radius: 50%;
        }
        .toggle-switch input:checked + .toggle-slider {
            background-color: var(--color-primary, #007bff);
        }
        .toggle-switch input:checked + .toggle-slider:before {
            transform: translateX(20px);
        }
        .toggle-switch input:focus + .toggle-slider {
            box-shadow: 0 0 1px var(--color-primary, #007bff);
        }
    };

    let path = config.path.clone();
    let enabled = config.enabled;
    let path_for_toggle = path.clone();
    let path_for_remove = path.clone();
    let path_for_import = path.clone();

    styled::view! { path_item_styles,
        <li class="path-item">
                <div class="toggle-group" style="flex: 1; margin-right: 10px;">
                    <label class="toggle-switch">
                        <input
                            type="checkbox"
                            checked=enabled
                            on:change=move |_| on_toggle(path_for_toggle.clone(), !enabled)
                        />
                        <span class="toggle-slider"></span>
                    </label>
                    <span class="path-text">{path}</span>
                </div>
                <div style="display: flex; gap: 8px;">
                    <button
                        class="button button-import"
                        on:click=move |_| on_import(path_for_import.clone())
                    >
                        <Icon icon=LuDownload width="16" height="16" />
                    </button>
                    <button
                        class="button button-danger"
                        on:click=move |_| on_remove(path_for_remove.clone())
                    >
                        <Icon icon=LuTrash2 width="16" height="16" />
                    </button>
                </div>
        </li>
    }
}

#[component]
fn ProgressOrError(
    importing_path: Option<String>,
    error_message: Option<String>,
) -> impl IntoView {
    let progress_styles = style! {
        .progress-container {
            margin-top: 8px;
            padding: 8px 12px;
            background-color: var(--color-bg-secondary, #f8f9fa);
            border: 1px solid var(--color-border, #dee2e6);
            border-radius: 4px;
        }
        .progress-bar-container {
            width: 100%;
            height: 8px;
            background-color: var(--color-border, #dee2e6);
            border-radius: 4px;
            overflow: hidden;
            margin-top: 8px;
            position: relative;
        }
        .progress-bar {
            height: 100%;
            width: 100%;
            background-color: var(--color-primary, #007bff);
            border-radius: 4px;
            opacity: 0.8;
        }
        .progress-text {
            font-size: 13px;
            color: var(--color-text-secondary, #6c757d);
            margin-bottom: 4px;
        }
        .error-message {
            margin-top: 8px;
            padding: 8px 12px;
            background-color: var(--color-error-bg, #fee);
            color: var(--color-error-text, #c33);
            border: 1px solid var(--color-error-border, #fcc);
            border-radius: 4px;
            font-size: 13px;
        }
    };

    if let Some(path) = importing_path {
        styled::view! { progress_styles,
            <div class="progress-container">
                <div class="progress-text">
                    {format!("Importing files from {}...", path)}
                </div>
                <div class="progress-bar-container">
                    <div class="progress-bar" style="width: 100%; animation: pulse 1.5s ease-in-out infinite;"></div>
                </div>
                <style>
                    {r#"
                    @keyframes pulse {
                        0%, 100% { opacity: 0.6; }
                        50% { opacity: 1; }
                    }
                    "#}
                </style>
            </div>
        }.into_any()
    } else if let Some(msg) = error_message {
        styled::view! { progress_styles,
            <div class="error-message">
                {msg}
            </div>
        }.into_any()
    } else {
        view! {}.into_any()
    }
}

#[component]
fn AddPathInput(
    new_path: RwSignal<String>,
    on_add: impl Fn() + Clone + 'static,
) -> impl IntoView {
    let add_path_styles = style! {
        .add-path-group {
            display: flex;
            gap: 8px;
            margin-top: 10px;
        }
        .add-path-input {
            flex: 1;
        }
        .form-control {
            width: 100%;
            padding: 8px 12px;
            border: 1px solid var(--color-border, #ccc);
            border-radius: 4px;
            background-color: var(--color-bg);
            color: var(--color-text);
            font-size: 14px;
        }
        .form-control:focus {
            outline: none;
            border-color: var(--color-primary, #007bff);
        }
        .button {
            padding: 8px 16px;
            border: none;
            border-radius: 4px;
            cursor: pointer;
            font-size: 14px;
            font-weight: 500;
            transition: background-color 0.2s;
            display: flex;
            align-items: center;
            justify-content: center;
            gap: 6px;
        }
        .button-primary {
            background-color: var(--color-primary, #007bff);
            color: white;
        }
        .button-primary:hover {
            background-color: var(--color-primary-hover, #0056b3);
        }
        .button-icon-only {
            padding: 8px;
        }
        .button-primary.button-icon-only {
            padding: 8px;
            width: 32px;
            height: 32px;
            min-width: 32px;
        }
    };

    styled::view! { add_path_styles,
        <div class="add-path-group">
                <input
                    type="text"
                    class="form-control add-path-input"
                    placeholder="Enter path to watch (e.g., ~/Downloads/Pic)"
                    prop:value=move || new_path.get()
                    on:input=move |ev| {
                        if let Some(target) = ev.target() {
                            if let Ok(input) = target.dyn_into::<web_sys::HtmlInputElement>() {
                                new_path.set(input.value());
                            }
                        }
                    }
                    on:keypress={
                        let on_add_clone = on_add.clone();
                        move |ev| {
                            if ev.key_code() == 13 {
                                on_add_clone();
                            }
                        }
                    }
                />
                <button class="button button-primary button-icon-only" on:click=move |_| on_add()>
                    <Icon icon=LuPlus width="16" height="16" />
                </button>
        </div>
    }
}

#[component]
fn CloseButton(close_settings: impl Fn() + 'static) -> impl IntoView {
    let close_button_styles = style! {
        .close-button {
            position: fixed;
            top: 20px;
            left: 20px;
            padding: 8px;
            background-color: var(--color-secondary, #6c757d);
            color: white;
            border: none;
            border-radius: 4px;
            cursor: pointer;
            font-size: 14px;
            font-weight: 500;
            transition: background-color 0.2s;
            z-index: 1001;
            display: flex;
            align-items: center;
            justify-content: center;
        }
        .close-button:hover {
            background-color: var(--color-secondary-hover, #5a6268);
        }
    };

    styled::view! { close_button_styles,
        <button class="close-button" on:click=move |_| close_settings()>
            <Icon icon=LuX width="20" height="20" />
        </button>
    }
}

#[component]
pub fn Settings(
    close_settings: impl Fn() + 'static,
    raw_entries_signal: RwSignal<Vec<miorin_core::prelude::Raw>>,
) -> impl IntoView {
    let container_styles = style! {
        .settings-wrapper {
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            width: 100%;
            height: 100vh;
            overflow-y: auto;
            background-color: var(--color-bg);
            z-index: 1000;
        }
        .settings-container {
            padding: 60px 20px 20px 20px;
            max-width: 800px;
            margin: 0 auto;
            background-color: var(--color-bg);
            color: var(--color-text);
            font-family: system-ui, -apple-system, sans-serif;
        }
        .section {
            margin-bottom: 30px;
        }
        .section-title {
            font-size: 18px;
            font-weight: 600;
            margin-bottom: 15px;
            color: var(--color-text);
        }
        .section-description {
            margin-bottom: 15px;
            color: var(--color-text-secondary, #6c757d);
            font-size: 13px;
        }
        .path-list {
            list-style: none;
            padding: 0;
            margin: 0;
        }
        .debug-section {
            margin-top: 40px;
            padding-top: 30px;
            border-top: 1px solid var(--color-border, #dee2e6);
        }
        .debug-button {
            padding: 10px 20px;
            background-color: var(--color-danger, #dc3545);
            color: white;
            border: none;
            border-radius: 4px;
            cursor: pointer;
            font-size: 14px;
            font-weight: 500;
            transition: background-color 0.2s;
            display: flex;
            align-items: center;
            gap: 8px;
        }
        .debug-button:hover {
            background-color: var(--color-danger-hover, #c82333);
        }
        .debug-button:disabled {
            background-color: var(--color-text-secondary, #6c757d);
            cursor: not-allowed;
        }
        .toggle-switch {
            position: relative;
            display: inline-block;
            width: 44px;
            height: 24px;
        }
        .toggle-switch input {
            opacity: 0;
            width: 0;
            height: 0;
        }
        .toggle-slider {
            position: absolute;
            cursor: pointer;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background-color: var(--color-toggle-bg, #ccc);
            transition: 0.3s;
            border-radius: 24px;
        }
        .toggle-slider:before {
            position: absolute;
            content: "";
            height: 18px;
            width: 18px;
            left: 3px;
            bottom: 3px;
            background-color: white;
            transition: 0.3s;
            border-radius: 50%;
        }
        .toggle-switch input:checked + .toggle-slider {
            background-color: var(--color-primary, #007bff);
        }
        .toggle-switch input:checked + .toggle-slider:before {
            transform: translateX(20px);
        }
        .toggle-switch input:focus + .toggle-slider {
            box-shadow: 0 0 1px var(--color-primary, #007bff);
        }
    };

    // Watch paths state
    let watch_paths = RwSignal::new(Vec::<tauri_api::WatchPathConfig>::new());
    let watch_paths_clone = watch_paths.clone();

    // New path input
    let new_path = RwSignal::new(String::new());

    // Error message state
    let error_message = RwSignal::new(Option::<String>::None);

    // Import progress state (path being imported, or None if not importing)
    let importing_path = RwSignal::new(Option::<String>::None);

    // Debug: Clear all raw entries state
    let clearing_entries = RwSignal::new(false);
    
    // Debug: Show settings storage state
    let showing_settings_storage = RwSignal::new(false);
    let settings_storage_data = RwSignal::new(Option::<String>::None);

    // Load settings on mount
    spawn_local({
        let watch_paths_clone = watch_paths_clone.clone();
        async move {
            // Load watch paths
            match tauri_api::get_watch_paths().await {
                | Ok(paths) => {
                    watch_paths_clone.set(paths);
                }
                | Err(e) => {
                    web_sys::console::error_1(&format!("Failed to load watch paths: {}", e).into());
                }
            }
        }
    });

    // Add path handler
    let add_path = {
        let new_path = new_path.clone();
        let watch_paths = watch_paths.clone();
        let error_message = error_message.clone();
        move || {
            let path = new_path.get();
            if !path.is_empty() {
                // Clear previous error
                error_message.set(None);

                let watch_paths_clone = watch_paths.clone();
                let new_path_clone = new_path.clone();
                let error_message_clone = error_message.clone();
                spawn_local(async move {
                    match tauri_api::add_watch_path(path.clone()).await {
                        | Ok(canonical_path) => {
                            web_sys::console::log_1(
                                &format!("Successfully added path: {}", canonical_path).into(),
                            );
                            // Clear input immediately after successful add
                            new_path_clone.set(String::new());
                            // Also clear any previous error
                            error_message_clone.set(None);
                            // Reload paths
                            match tauri_api::get_watch_paths().await {
                                | Ok(paths) => {
                                    watch_paths_clone.set(paths);
                                }
                                | Err(e) => {
                                    error_message_clone
                                        .set(Some(format!("Failed to reload watch paths: {}", e)));
                                    web_sys::console::error_1(
                                        &format!("Failed to reload watch paths: {}", e).into(),
                                    );
                                }
                            }
                        }
                        | Err(e) => {
                            error_message_clone.set(Some(e.clone()));
                            web_sys::console::error_1(
                                &format!("Failed to add watch path: {}", e).into(),
                            );
                        }
                    }
                });
            }
        }
    };

    // Remove path handler
    let remove_path = move |path: String| {
        let watch_paths_clone = watch_paths.clone();
        spawn_local(async move {
            match tauri_api::remove_watch_path(path.clone()).await {
                | Ok(_) => {
                    // Reload paths
                    match tauri_api::get_watch_paths().await {
                        | Ok(paths) => {
                            watch_paths_clone.set(paths);
                        }
                        | Err(e) => {
                            web_sys::console::error_1(
                                &format!("Failed to reload watch paths: {}", e).into(),
                            );
                        }
                    }
                }
                | Err(e) => {
                    web_sys::console::error_1(
                        &format!("Failed to remove watch path: {}", e).into(),
                    );
                }
            }
        });
    };

    // Toggle path enabled handler
    let toggle_path_enabled = move |path: String, enabled: bool| {
        let watch_paths_clone = watch_paths.clone();
        spawn_local(async move {
            match tauri_api::toggle_watch_path_enabled(path.clone(), enabled).await {
                | Ok(_) => {
                    // Reload paths
                    match tauri_api::get_watch_paths().await {
                        | Ok(paths) => {
                            watch_paths_clone.set(paths);
                        }
                        | Err(e) => {
                            web_sys::console::error_1(
                                &format!("Failed to reload watch paths: {}", e).into(),
                            );
                        }
                    }
                }
                | Err(e) => {
                    web_sys::console::error_1(
                        &format!("Failed to toggle watch path: {}", e).into(),
                    );
                }
            }
        });
    };

    // Import files handler
    let import_files = {
        let importing_path_clone = importing_path.clone();
        let error_message_clone = error_message.clone();
        let raw_entries_signal_clone = raw_entries_signal.clone();
        move |path: String| {
            let importing_path_inner = importing_path_clone.clone();
            let error_message_inner = error_message_clone.clone();
            let raw_entries_signal_inner = raw_entries_signal_clone.clone();
            spawn_local(async move {
                error_message_inner.set(None);
                importing_path_inner.set(Some(path.clone()));
                match tauri_api::import_files_from_path(path.clone()).await {
                    | Ok(count) => {
                        web_sys::console::log_1(
                            &format!("Successfully imported {} files from {}", count, path).into(),
                        );
                        importing_path_inner.set(None);
                        if count > 0 {
                            error_message_inner.set(Some(format!("Successfully imported {} files", count)));
                            // Refresh raw entries to show newly imported files
                            match tauri_api::get_all_raw_entries().await {
                                | Ok(entries) => {
                                    web_sys::console::log_1(
                                        &format!("Successfully refreshed {} raw entries", entries.len()).into(),
                                    );
                                    raw_entries_signal_inner.set(entries);
                                }
                                | Err(e) => {
                                    web_sys::console::error_1(&format!("Failed to refresh raw entries: {}", e).into());
                                }
                            }
                        } else {
                            error_message_inner.set(Some("No files found to import".to_string()));
                        }
                    }
                    | Err(e) => {
                        importing_path_inner.set(None);
                        error_message_inner.set(Some(format!("Failed to import files: {}", e)));
                        web_sys::console::error_1(
                            &format!("Failed to import files from path: {}", e).into(),
                        );
                    }
                }
            });
        }
    };

    styled::view! { container_styles,
        <div class="settings-wrapper">
            <CloseButton close_settings=close_settings />
            <div class="settings-container">
                <div class="section">
                    <h2 class="section-title">Watch Paths</h2>
                    <p class="section-description">
                        Directories to watch for new files.
                        Only files directly in these directories (not in subdirectories) will be monitored.
                        Toggle each path to enable or disable watching.
                    </p>
                    <ul class="path-list">
                        <For
                            each=move || watch_paths.get()
                            key=|config| config.path.clone()
                            children=move |config: tauri_api::WatchPathConfig| {
                                view! {
                                    <WatchPathItem
                                        config=config.clone()
                                        on_toggle=toggle_path_enabled.clone()
                                        on_remove=remove_path.clone()
                                        on_import=import_files.clone()
                                    />
                                }
                            }
                        />
                    </ul>
                    <AddPathInput new_path=new_path on_add=add_path.clone() />
                    {move || {
                        view! {
                            <ProgressOrError
                                importing_path=importing_path.get()
                                error_message=error_message.get()
                            />
                        }.into_any()
                    }}
                </div>
                <div class="section debug-section">
                    <h2 class="section-title">Debug</h2>
                    <p class="section-description">
                        Debug functions for development and testing.
                    </p>
                    <div style="display: flex; gap: 10px; flex-wrap: wrap;">
                        {move || {
                            let clearing_entries_value = clearing_entries.get();
                            let clearing_entries_signal = clearing_entries.clone();
                            view! {
                                <button
                                    class="debug-button"
                                    disabled=clearing_entries_value
                                    on:click=move |_| {
                                        clearing_entries_signal.set(true);
                                        let clearing_entries_signal_clone = clearing_entries_signal.clone();
                                        spawn_local(async move {
                                            match tauri_api::delete_all_raw_entries().await {
                                                | Ok(count) => {
                                                    web_sys::console::log_1(
                                                        &format!("Deleted {} raw entries", count).into(),
                                                    );
                                                    // Reload the page to refresh the UI
                                                    web_sys::window()
                                                        .and_then(|w| w.location().reload().ok());
                                                }
                                                | Err(e) => {
                                                    web_sys::console::error_1(
                                                        &format!("Failed to delete all raw entries: {}", e).into(),
                                                    );
                                                    clearing_entries_signal_clone.set(false);
                                                }
                                            }
                                        });
                                    }
                                >
                                    <Icon icon=LuTrash2 width="16" height="16" />
                                    {if clearing_entries_value { "Clearing..." } else { "Clear All Raw Entries" }}
                                </button>
                            }
                        }}
                        {move || {
                            let showing_settings_storage_value = showing_settings_storage.get();
                            let showing_settings_storage_signal = showing_settings_storage.clone();
                            let settings_storage_data_signal = settings_storage_data.clone();
                            let has_data = settings_storage_data.get().is_some();
                            view! {
                                <button
                                    class="debug-button"
                                    disabled=showing_settings_storage_value
                                    on:click=move |_| {
                                        // Toggle: if data is already shown, hide it
                                        let current_data = settings_storage_data_signal.get();
                                        if current_data.is_some() {
                                            settings_storage_data_signal.set(None);
                                        } else {
                                            // Otherwise, fetch and show data
                                            showing_settings_storage_signal.set(true);
                                            settings_storage_data_signal.set(None);
                                            let showing_settings_storage_signal_clone = showing_settings_storage_signal.clone();
                                            let settings_storage_data_signal_clone = settings_storage_data_signal.clone();
                                            spawn_local(async move {
                                                match tauri_api::get_all_store_data().await {
                                                    | Ok(data) => {
                                                        let json_string = serde_json::to_string_pretty(&data)
                                                            .unwrap_or_else(|_| format!("{:?}", data));
                                                        settings_storage_data_signal_clone.set(Some(json_string));
                                                        showing_settings_storage_signal_clone.set(false);
                                                    }
                                                    | Err(e) => {
                                                        web_sys::console::error_1(
                                                            &format!("Failed to get store data: {}", e).into(),
                                                        );
                                                        settings_storage_data_signal_clone.set(Some(format!("Error: {}", e)));
                                                        showing_settings_storage_signal_clone.set(false);
                                                    }
                                                }
                                            });
                                        }
                                    }
                                >
                                    <Icon icon=if has_data { LuEyeOff } else { LuEye } width="16" height="16" />
                                    {if showing_settings_storage_value { "Loading..." } else if has_data { "Hide Settings Storage State" } else { "Show Settings Storage State" }}
                                </button>
                            }
                        }}
                        {move || {
                            view! {
                                <button
                                    class="debug-button"
                                    on:click=move |_| {
                                        // Force refresh by reloading the page
                                        web_sys::window()
                                            .and_then(|w| w.location().reload().ok());
                                    }
                                >
                                    <Icon icon=LuRefreshCw width="16" height="16" />
                                    "Force Refresh"
                                </button>
                            }
                        }}
                        {move || {
                            view! {
                                <button
                                    class="debug-button"
                                    on:click=move |_| {
                                        spawn_local(async move {
                                            match tauri_api::reveal_data_folder().await {
                                                | Ok(path) => {
                                                    web_sys::console::log_1(
                                                        &format!("Opened data folder: {}", path).into(),
                                                    );
                                                }
                                                | Err(e) => {
                                                    web_sys::console::error_1(
                                                        &format!("Failed to open data folder: {}", e).into(),
                                                    );
                                                }
                                            }
                                        });
                                    }
                                >
                                    <Icon icon=LuFolderOpen width="16" height="16" />
                                    "Reveal Data Folder on Device"
                                </button>
                            }
                        }}
                        {move || {
                            view! {
                                <button
                                    class="debug-button"
                                    on:click=move |_| {
                                        spawn_local(async move {
                                            if let Err(e) = tauri_api::toggle_devtools().await {
                                                web_sys::console::error_1(
                                                    &format!("Failed to open devtools: {}", e).into(),
                                                );
                                            }
                                        });
                                    }
                                >
                                    <Icon icon=LuEye width="16" height="16" />
                                    "Open Developer Console"
                                </button>
                            }
                        }}
                    </div>
                    {move || {
                        let settings_storage_data_value = settings_storage_data.get();
                        settings_storage_data_value.map(|data| {
                            let data_clone = data.clone();
                            view! {
                                <div style="margin-top: 20px; background-color: transparent; border-radius: 4px; border: 1px solid var(--color-border, #dee2e6);">
                                    <h3 style="margin: 0; padding: 15px 15px 10px 15px; font-size: 14px; font-weight: 600;">Settings Storage State:</h3>
                                    <pre style="margin: 0; padding: 0 15px 15px 15px; background-color: var(--color-bg, #ffffff); overflow-x: auto; font-family: monospace; font-size: 12px; white-space: pre-wrap; word-wrap: break-word; overflow-wrap: break-word; line-height: 1.4;">{data_clone}</pre>
                                </div>
                            }
                        })
                    }}
                </div>
            </div>
        </div>
    }
}
