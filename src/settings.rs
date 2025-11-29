use crate::tauri_api;
use crate::button;
use crate::color::ColorVariant;
use crate::toggle;
use leptos::prelude::*;
use leptos_icons::Icon;
use icondata::{LuDownload, LuEye, LuEyeOff, LuFolderOpen, LuPlus, LuRefreshCw, LuTrash2, LuX};
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen::{JsCast, closure::Closure};
use std::rc::Rc;


#[component]
fn WatchPathItem(
    config: tauri_api::WatchPathConfig,
    on_toggle: impl Fn(String, bool) + Clone + Send + 'static,
    on_remove: impl Fn(String) + Clone + Send + 'static,
    on_import: impl Fn(String) + Clone + Send + 'static,
) -> impl IntoView {
    let path = config.path.clone();
    let enabled_signal = RwSignal::new(config.enabled);
    let path_for_toggle = path.clone();
    let path_for_action = path.clone();

    view! {
        <li 
            class="flex items-center justify-between p-2.5 mb-2 rounded border bg-[var(--color-bg-secondary)] border-[var(--color-border)]"
        >
            <div class="flex items-center gap-2 flex-1 mr-2.5">
                <toggle::Toggle
                    enabled=enabled_signal
                    on_change=move |new_value| {
                        on_toggle(path_for_toggle.clone(), new_value);
                    }
                />
                <span class="flex-1 break-all text-xs font-mono text-[var(--color-text-secondary,#6c757d)]">{path}</span>
            </div>
            <div class="flex gap-2">
                {move || {
                    let path_for_action = path_for_action.clone();
                    if enabled_signal.get() {
                        let on_import = on_import.clone();
                        view! {
                            <button::ActionButton
                                icon=view! { <Icon icon=LuDownload width="12" height="12" /> }
                                color_variant=ColorVariant::Primary
                                on_click=move |_| on_import(path_for_action.clone())
                            />
                        }.into_any()
                    } else {
                        let on_remove = on_remove.clone();
                        view! {
                            <button::ActionButton
                                icon=view! { <Icon icon=LuTrash2 width="12" height="12" /> }
                                color_variant=ColorVariant::Danger
                                on_click=move |_| on_remove(path_for_action.clone())
                            />
                        }.into_any()
                    }
                }}
            </div>
        </li>
    }
}

#[component]
fn ProgressOrError(
    importing_path: Option<String>,
    error_message: Option<String>,
) -> impl IntoView {
    if let Some(path) = importing_path {
        view! {
            <div 
                class="mt-2 p-2 rounded border bg-[var(--color-bg-secondary,#f8f9fa)] border-[var(--color-border,#dee2e6)]"
            >
                <div class="text-xs mb-1 text-[var(--color-text-secondary,#6c757d)]">
                    {format!("Importing files from {}...", path)}
                </div>
                <div 
                    class="w-full h-2 rounded overflow-hidden mt-2 relative bg-[var(--color-border,#dee2e6)]"
                >
                    <div 
                        class="h-full w-full rounded opacity-80 animate-pulse bg-[var(--color-primary,#007bff)]"
                    ></div>
                </div>
            </div>
        }.into_any()
    } else if let Some(msg) = error_message {
        view! {
            <div 
                class="mt-2 p-2 rounded border text-xs bg-[var(--color-error-bg,#fee)] text-[var(--color-error-text,#c33)] border-[var(--color-error-border,#fcc)]"
            >
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
    let on_add_clone = on_add.clone();
    view! {
        <div class="flex gap-2 mt-2.5">
            <button::InterfaceButton
                icon=view! { <Icon icon=LuPlus width="16" height="16" /> }
                color_variant=ColorVariant::Primary
                on_click=move |_| on_add_clone()
                class="pl-2"
            />
            <button::InterfaceButton
                icon=view! { <Icon icon=LuFolderOpen width="16" height="16" /> }
                color_variant=ColorVariant::Secondary
                on_click=move |_| {
                    let new_path_clone = new_path.clone();
                    spawn_local(async move {
                        match tauri_api::open_folder_dialog().await {
                            Ok(Some(path)) => {
                                new_path_clone.set(path);
                            }
                            Ok(None) => {
                                // User cancelled the dialog
                            }
                            Err(e) => {
                                web_sys::console::error_1(&format!("Failed to open folder dialog: {}", e).into());
                            }
                        }
                    });
                }
            />
            <input
                type="text"
                class="flex-1 w-full px-3 py-2 border rounded text-sm outline-none border-[var(--color-border,#ccc)] bg-[var(--color-bg)] text-[var(--color-text)]"
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
                    move |ev| {
                        // If Enter key is pressed, add the path
                        if ev.key_code() == 13 {
                            on_add();
                        }
                    }
                }
            />
        </div>
    }
}

#[component]
fn CloseButton(close_settings: impl Fn() + 'static) -> impl IntoView {
    view! {
        <div class="fixed top-5 left-5 z-[1001]">
            <button::InterfaceButton
                icon=view! { <Icon icon=LuX width="20" height="20" /> }
                color_variant=ColorVariant::Secondary
                on_click=move |_| close_settings()
            />
        </div>
    }
}

#[component]
pub fn Settings(
    close_settings: impl Fn() + 'static,
    raw_entries_signal: RwSignal<Vec<miorin_core::prelude::Raw>>,
) -> impl IntoView {

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

    // Listen for ESC key to close settings
    // Box the closure so we can store it and use it in multiple places
    let close_settings_boxed: Rc<dyn Fn()> = Rc::new(move || close_settings());
    {
        let close_settings_clone = close_settings_boxed.clone();
        spawn_local(async move {
            let handler = Closure::wrap(Box::new(move |ev: web_sys::KeyboardEvent| {
                if ev.key_code() == 27 {
                    // ESC key pressed
                    close_settings_clone();
                }
            }) as Box<dyn FnMut(_)>);
            
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            document.add_event_listener_with_callback(
                "keydown",
                handler.as_ref().unchecked_ref()
            ).unwrap();
            handler.forget();
        });
    }

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

    view! {
        <div 
            class="fixed top-0 left-0 right-0 bottom-0 w-full h-screen overflow-y-auto z-[1000] bg-[var(--color-bg)]"
        >
            <CloseButton close_settings=move || close_settings_boxed() />
            <div 
                class="pt-[60px] px-5 pb-5 max-w-[800px] mx-auto bg-[var(--color-bg)] text-[var(--color-text)] font-[system-ui,-apple-system,sans-serif]"
            >
                <div class="mb-8">
                    <h2 class="text-lg font-semibold mb-4 text-[var(--color-text)]">Watch Paths</h2>
                    <p class="mb-4 text-xs text-[var(--color-text-secondary,#6c757d)]">
                        Directories to watch for new files.
                        Only files directly in these directories (not in subdirectories) will be monitored.
                        Toggle each path to enable or disable watching.
                    </p>
                    <ul class="list-none p-0 m-0">
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
                <div class="mb-8 mt-10 pt-8 border-t border-[var(--color-border,#dee2e6)]">
                    <h2 class="text-lg font-semibold mb-4 text-[var(--color-text)]">Debug</h2>
                    <p class="mb-4 text-xs text-[var(--color-text-secondary,#6c757d)]">
                        Debug functions for development and testing.
                    </p>
                    <div class="flex gap-2.5 flex-wrap">
                        {move || {
                            let clearing_entries_value = clearing_entries.get();
                            let clearing_entries_signal = clearing_entries.clone();
                            view! {
                                <button
                                    class="px-5 py-2.5 text-white border-none rounded cursor-pointer text-sm font-medium transition-colors duration-200 flex items-center gap-2 disabled:cursor-not-allowed hover-bg-danger disabled:hover-bg-secondary"
                                    style=move || format!(
                                        "background-color: {};",
                                        if clearing_entries_value { "var(--color-text-secondary, #6c757d)" } else { "var(--color-danger, #dc3545)" }
                                    )
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
                                    class="px-5 py-2.5 text-white border-none rounded cursor-pointer text-sm font-medium transition-colors duration-200 flex items-center gap-2 disabled:cursor-not-allowed hover-bg-danger disabled:hover-bg-secondary"
                                    style=move || format!(
                                        "background-color: {};",
                                        if showing_settings_storage_value { "var(--color-text-secondary, #6c757d)" } else { "var(--color-danger, #dc3545)" }
                                    )
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
                                    class="px-5 py-2.5 text-white border-none rounded cursor-pointer text-sm font-medium transition-colors duration-200 flex items-center gap-2 hover-bg-danger bg-[var(--color-danger,#dc3545)]"
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
                                    class="px-5 py-2.5 text-white border-none rounded cursor-pointer text-sm font-medium transition-colors duration-200 flex items-center gap-2 hover-bg-danger bg-[var(--color-danger,#dc3545)]"
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
                                    class="px-5 py-2.5 text-white border-none rounded cursor-pointer text-sm font-medium transition-colors duration-200 flex items-center gap-2 hover-bg-danger bg-[var(--color-danger,#dc3545)]"
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
                                <div class="mt-5 bg-transparent rounded border p-0 border-[var(--color-border,#dee2e6)]">
                                    <h3 class="m-0 px-4 pt-4 pb-2.5 text-sm font-semibold">Settings Storage State:</h3>
                                    <pre class="m-0 px-4 pb-4 overflow-x-auto font-mono text-xs whitespace-pre-wrap break-words leading-snug bg-[var(--color-bg,#ffffff)]">{data_clone}</pre>
                                </div>
                            }
                        })
                    }}
                </div>
            </div>
        </div>
    }
}
