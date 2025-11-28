use crate::tauri_api;
use leptos::prelude::*;
use styled::style;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen::JsCast;

#[component]
pub fn Settings(close_settings: impl Fn() + 'static) -> impl IntoView {
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
        .form-group {
            margin-bottom: 15px;
        }
        .form-label {
            display: block;
            margin-bottom: 5px;
            font-weight: 500;
            color: var(--color-text);
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
        .button {
            padding: 8px 16px;
            border: none;
            border-radius: 4px;
            cursor: pointer;
            font-size: 14px;
            font-weight: 500;
            transition: background-color 0.2s;
        }
        .button-primary {
            background-color: var(--color-primary, #007bff);
            color: white;
        }
        .button-primary:hover {
            background-color: var(--color-primary-hover, #0056b3);
        }
        .button-danger {
            background-color: var(--color-danger, #dc3545);
            color: white;
        }
        .button-danger:hover {
            background-color: var(--color-danger-hover, #c82333);
        }
        .button-secondary {
            background-color: var(--color-secondary, #6c757d);
            color: white;
        }
        .button-secondary:hover {
            background-color: var(--color-secondary-hover, #5a6268);
        }
        .path-list {
            list-style: none;
            padding: 0;
            margin: 0;
        }
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
        .add-path-group {
            display: flex;
            gap: 8px;
            margin-top: 10px;
        }
        .add-path-input {
            flex: 1;
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

    // Watch paths state
    let watch_paths = RwSignal::new(Vec::<tauri_api::WatchPathConfig>::new());
    let watch_paths_clone = watch_paths.clone();

    // New path input
    let new_path = RwSignal::new(String::new());

    // Error message state
    let error_message = RwSignal::new(Option::<String>::None);

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

    let close_button_styles = style! {
        .close-button {
            position: fixed;
            top: 20px;
            left: 20px;
            padding: 8px 16px;
            background-color: var(--color-secondary, #6c757d);
            color: white;
            border: none;
            border-radius: 4px;
            cursor: pointer;
            font-size: 14px;
            font-weight: 500;
            transition: background-color 0.2s;
            z-index: 1001;
        }
        .close-button:hover {
            background-color: var(--color-secondary-hover, #5a6268);
        }
    };

    styled::view! { container_styles,
        <div class="settings-wrapper">
            {styled::view! { close_button_styles,
                <button class="close-button" on:click=move |_| close_settings()>
                    "Close"
                </button>
            }}
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
                                let path = config.path.clone();
                                let enabled = config.enabled;
                                let path_for_toggle = path.clone();
                                let path_for_remove = path.clone();
                                view! {
                                    <li class="path-item">
                                        <div class="toggle-group" style="flex: 1; margin-right: 10px;">
                                            <label class="toggle-switch">
                                                <input
                                                    type="checkbox"
                                                    checked=enabled
                                                    on:change=move |_| toggle_path_enabled(path_for_toggle.clone(), !enabled)
                                                />
                                                <span class="toggle-slider"></span>
                                            </label>
                                            <span class="path-text">{path}</span>
                                        </div>
                                        <button
                                            class="button button-danger"
                                            on:click=move |_| remove_path(path_for_remove.clone())
                                        >
                                            Remove
                                        </button>
                                    </li>
                                }
                            }
                        />
                    </ul>
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
                            on:keypress=move |ev| {
                                if ev.key_code() == 13 {
                                    add_path();
                                }
                            }
                        />
                        <button class="button button-primary" on:click=move |_| add_path()>
                            Add Path
                        </button>
                    </div>
                    {move || {
                        error_message.get().map(|msg| {
                            view! {
                                <div class="error-message">
                                    {msg}
                                </div>
                            }
                        })
                    }}
                </div>
            </div>
        </div>
    }
}
