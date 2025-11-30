use crate::ui::{
    button, card,
    color::ColorVariant,
    filter::{FilterButton, FilterMode},
    panel,
};
use crate::tauri_api;
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_use::use_debounce_fn_with_arg;
use icondata::{LuPlus, LuSettings, LuTrash2};
use miorin_core::prelude::*;
use wasm_bindgen_futures::spawn_local;

#[component]
pub fn App() -> impl IntoView {
    // Load raw entries from database
    let raw_entries = RwSignal::new(Vec::<Raw>::new());
    let raw_entries_for_load = raw_entries.clone();
    spawn_local(async move {
        web_sys::console::log_1(&"Starting to load raw entries from database...".into());
        match tauri_api::get_all_raw_entries().await {
            | Ok(entries) => {
                web_sys::console::log_1(
                    &format!("Successfully loaded {} raw entries", entries.len()).into(),
                );
                raw_entries_for_load.set(entries);
            }
            | Err(e) => {
                web_sys::console::error_1(&format!("Failed to load raw entries: {}", e).into());
            }
        }
        web_sys::console::log_1(&"Finished loading raw entries".into());
    });

    // Listen for raw-entry-created events to refresh the stream panel
    let raw_entries_for_event = raw_entries.clone();
    spawn_local(async move {
        if let Err(e) = tauri_api::listen_to_event("raw-entry-created", move |_event| {
            web_sys::console::log_1(&"Received raw-entry-created event".into());
            let raw_entries = raw_entries_for_event.clone();
            spawn_local(async move {
                web_sys::console::log_1(&"Refreshing raw entries after new entry created...".into());
                match tauri_api::get_all_raw_entries().await {
                    | Ok(entries) => {
                        web_sys::console::log_1(
                            &format!("Refreshed raw entries, now have {} entries", entries.len()).into(),
                        );
                        raw_entries.set(entries);
                    }
                    | Err(e) => {
                        web_sys::console::error_1(&format!("Failed to refresh raw entries: {}", e).into());
                    }
                }
            });
        }) {
            web_sys::console::error_1(&format!("Failed to listen to raw-entry-created event: {}", e).into());
        } else {
            web_sys::console::log_1(&"Successfully set up listener for raw-entry-created event".into());
        }
    });

    // Load cubes from database
    let cubes = RwSignal::new(Vec::<Cube>::new());
    spawn_local(async move {
        match tauri_api::get_all_cubes().await {
            | Ok(cubes_data) => cubes.set(cubes_data),
            | Err(e) => {
                web_sys::console::error_1(&format!("Failed to load cubes: {}", e).into());
            }
        }
    });

    // Settings visibility state
    let show_settings = RwSignal::new(false);

    // Panel widths - load once at app level so they persist across view switches
    let left_width = RwSignal::new(250.0);
    let right_width = RwSignal::new(300.0);
    let left_width = left_width.clone();
    let right_width = right_width.clone();
    spawn_local(async move {
        match tauri_api::get_panel_left_width().await {
            | Ok(width) => left_width.set(width),
            | Err(e) => {
                web_sys::console::warn_1(&format!("Failed to load left panel width: {}", e).into());
            }
        }
        match tauri_api::get_panel_right_width().await {
            | Ok(width) => right_width.set(width),
            | Err(e) => {
                web_sys::console::warn_1(
                    &format!("Failed to load right panel width: {}", e).into(),
                );
            }
        }
    });

    // Settings button handler
    let open_settings = {
        let show_settings = show_settings.clone();
        move |_| {
            show_settings.set(true);
        }
    };

    let close_settings = {
        let show_settings = show_settings.clone();
        move || {
            show_settings.set(false);
        }
    };

    view! {
        {move || {
            if show_settings.get() {
                view! {
                    <crate::settings::Settings
                        close_settings=close_settings.clone()
                        raw_entries_signal=raw_entries.clone()
                    />
                }.into_any()
            } else {
                view! {
                    <MainView
                        cubes=cubes
                        raw_entries=raw_entries
                        open_settings=open_settings
                        left_width=left_width
                        right_width=right_width
                    />
                }.into_any()
            }
        }}
    }
}

#[component]
fn MainView(
    cubes: RwSignal<Vec<Cube>>, raw_entries: RwSignal<Vec<Raw>>,
    open_settings: impl Fn(web_sys::MouseEvent) + 'static, left_width: RwSignal<f64>,
    right_width: RwSignal<f64>,
) -> impl IntoView {
    // Debounced save function for left panel width
    let save_left_width = move |width: f64| {
        spawn_local(async move {
            if let Err(e) = tauri_api::set_panel_left_width(width).await {
                web_sys::console::error_1(
                    &format!("Failed to save left panel width: {}", e).into(),
                );
            }
        });
    };
    let debounced_save_left = use_debounce_fn_with_arg(save_left_width, 500.0);

    // Debounced save function for right panel width
    let save_right_width = move |width: f64| {
        spawn_local(async move {
            if let Err(e) = tauri_api::set_panel_right_width(width).await {
                web_sys::console::error_1(
                    &format!("Failed to save right panel width: {}", e).into(),
                );
            }
        });
    };
    let debounced_save_right = use_debounce_fn_with_arg(save_right_width, 500.0);

    let on_resize_left = {
        let left_width = left_width.clone();
        let debounced_save_left = debounced_save_left.clone();
        std::rc::Rc::new(move |new_width: f64| {
            left_width.set(new_width);
            debounced_save_left(new_width);
        }) as std::rc::Rc<dyn Fn(f64)>
    };

    let on_resize_right = {
        let right_width = right_width.clone();
        let debounced_save_right = debounced_save_right.clone();
        std::rc::Rc::new(move |new_width: f64| {
            right_width.set(new_width);
            debounced_save_right(new_width);
        }) as std::rc::Rc<dyn Fn(f64)>
    };

    view! {
        <div
            class="grid h-screen max-h-screen overflow-hidden bg-[var(--color-bg)]"
            style=move || format!("grid-template-columns: {}px 1fr {}px;", left_width.get(), right_width.get())
        >
            <GlacierPanel
                cubes=cubes
                on_resize_right=on_resize_left.clone()
            />
            <Workspace />
            <StreamPanel
                raw_entries=raw_entries
                open_settings=open_settings
                on_resize_left=on_resize_right.clone()
            />
        </div>
    }
}

#[component]
fn GlacierPanel(
    cubes: RwSignal<Vec<Cube>>, on_resize_right: std::rc::Rc<dyn Fn(f64) + 'static>,
) -> impl IntoView {
    let create_cube_action = move |_| {
        let cubes_signal = cubes.clone();
        spawn_local(async move {
            // Create a dummy point cube with paragraph text
            let dummy_point = Point::Text(Text {
                style: TextStyle::Paragraph,
                text: RichText {
                    segments: vec![RichTextSegment {
                        text: "Text".to_string(),
                        marks: TextMarks::default(),
                    }],
                },
            });

            web_sys::console::log_1(&"Creating cube...".into());
            match tauri_api::create_cube(true, CubeContent::Point(dummy_point)).await {
                | Ok(cube) => {
                    web_sys::console::log_1(
                        &format!("Cube created successfully with id: {}", cube.id.0).into(),
                    );
                    // Reload cubes
                    match tauri_api::get_all_cubes().await {
                        | Ok(cubes_data) => {
                            web_sys::console::log_1(
                                &format!("Reloaded {} cubes", cubes_data.len()).into(),
                            );
                            cubes_signal.set(cubes_data);
                        }
                        | Err(e) => {
                            web_sys::console::error_1(
                                &format!("Failed to reload cubes: {}", e).into(),
                            );
                        }
                    }
                }
                | Err(e) => {
                    web_sys::console::error_1(&format!("Failed to create cube: {}", e).into());
                }
            }
        });
    };

    // Filter state
    let filter_mode = RwSignal::new(FilterMode::Pinned);

    view! {
        <panel::Panel
            title="Glacier"
            resizable_right=true
            on_resize_right=on_resize_right.clone()
            header_actions=view! {
                <div class="flex items-center gap-1.5">
                    <FilterButton filter_mode=filter_mode.clone() />
                    <button::ActionButton
                        icon=view! { <Icon icon=LuPlus width="12" height="12" /> }
                        color_variant=ColorVariant::Primary
                        on_click=create_cube_action
                    />
                </div>
            }.into_any()
        >
            <card::CardList items=move || {
                let cubes_data = cubes.get();
                let mode = filter_mode.get();
                web_sys::console::log_1(
                    &format!("GlacierPanel: Got {} total cubes, filter mode: {:?}", cubes_data.len(), mode).into(),
                );
                let filtered: Vec<_> = cubes_data
                    .into_iter()
                    .filter(|cube| {
                        match mode {
                            FilterMode::Pinned => cube.inner.pin,
                            FilterMode::All => true,
                        }
                    })
                    .collect();
                web_sys::console::log_1(
                    &format!("GlacierPanel: {} cubes after filter", filtered.len()).into(),
                );
                filtered
                    .into_iter()
                    .map(|cube| view! { <CubeEntry cube=cube cubes=cubes.clone() /> }.into_any())
                    .collect::<Vec<_>>()
            } />
        </panel::Panel>
    }
}

#[component]
fn Workspace() -> impl IntoView {
    view! {
        <div
            class="flex flex-col h-full border-r overflow-hidden border-[var(--color-border)] bg-[var(--color-workspace-bg)]"
        >
            <div
                class="py-2 pl-3 pr-2 border-b border-[var(--color-border)] bg-[var(--color-panel-header-bg)]"
            >
                <h2 class="m-0 text-sm font-medium text-[var(--color-text)]">"Workspace"</h2>
            </div>
            <div class="flex-1 overflow-y-auto p-2">
                <div class="p-8 min-h-full">
                    <p class="italic text-center mt-12 text-[var(--color-text-muted)]">
                        "Select a cube from Glacier or drag raw material from Stream to start editing..."
                    </p>
                </div>
                </div>
        </div>
    }
}

#[component]
fn StreamPanel(
    raw_entries: RwSignal<Vec<Raw>>, open_settings: impl Fn(web_sys::MouseEvent) + 'static,
    on_resize_left: std::rc::Rc<dyn Fn(f64) + 'static>,
) -> impl IntoView {
    view! {
        <panel::Panel
            title="Stream"
            resizable_left=true
            on_resize_left=on_resize_left.clone()
            header_actions=view! {
                <button::ActionButton
                    icon=view! { <Icon icon=LuSettings width="12" height="12" /> }
                    color_variant=ColorVariant::Secondary
                    on_click=open_settings
                />
            }.into_any()
        >
            <card::CardList items=move || {
                let entries = raw_entries.get();
                entries.into_iter()
                    .map(|raw| view! { <RawEntry raw=raw /> }.into_any())
                    .collect::<Vec<_>>()
            } />
        </panel::Panel>
    }
}

#[component]
fn CubeEntry(cube: Cube, cubes: RwSignal<Vec<Cube>>) -> impl IntoView {
    let title = match &cube.inner.content {
        | CubeContent::Graph(_) => "Graph".to_string(),
        | CubeContent::PreOrder(_) => "PreOrder".to_string(),
        | CubeContent::Order(_) => "Order".to_string(),
        | CubeContent::Point(_) => "Point".to_string(),
        | CubeContent::Meta => "Meta".to_string(),
    };
    let created = cube.created_at.format("%Y-%m-%d %H:%M").to_string();
    let cube_id = cube.id.0.to_string();
    
    let delete_action = {
        let cubes_signal = cubes.clone();
        let cube_id_clone = cube_id.clone();
        move |e: web_sys::MouseEvent| {
            e.stop_propagation();
            let cubes_signal = cubes_signal.clone();
            let cube_id = cube_id_clone.clone();
            spawn_local(async move {
                web_sys::console::log_1(&format!("Deleting cube with id: {}", cube_id).into());
                match tauri_api::delete_cube(cube_id).await {
                    | Ok(_) => {
                        web_sys::console::log_1(&"Cube deleted successfully".into());
                        // Reload cubes
                        match tauri_api::get_all_cubes().await {
                            | Ok(cubes_data) => {
                                web_sys::console::log_1(
                                    &format!("Reloaded {} cubes", cubes_data.len()).into(),
                                );
                                cubes_signal.set(cubes_data);
                            }
                            | Err(e) => {
                                web_sys::console::error_1(
                                    &format!("Failed to reload cubes: {}", e).into(),
                                );
                            }
                        }
                    }
                    | Err(e) => {
                        web_sys::console::error_1(&format!("Failed to delete cube: {}", e).into());
                    }
                }
            });
        }
    };
    
    view! {
        <div
            class="p-3 border rounded cursor-pointer transition-colors duration-200 hover-bg-panel border-[var(--color-border)] bg-[var(--color-panel-bg)] relative group"
        >
            <div class="flex items-start justify-between gap-2">
                <div class="flex-1">
                    <div class="font-medium mb-1 text-[var(--color-text)]">{title}</div>
                    <div class="text-xs mt-1 text-[var(--color-text-secondary)]">{created}</div>
                </div>
                <button::ActionButton
                    icon=view! { <Icon icon=LuTrash2 width="12" height="12" /> }
                    color_variant=ColorVariant::Danger
                    on_click=delete_action
                    class="opacity-0 group-hover:opacity-100 transition-opacity duration-200 shrink-0"
                />
            </div>
        </div>
    }
}

#[component]
fn RawEntry(raw: Raw) -> impl IntoView {
    let source = format_raw_source(&raw.inner.source);
    let created = raw.created_at.format("%Y-%m-%d %H:%M").to_string();
    let content = raw.inner.content.clone();
    view! {
        <div
            class="p-3 border rounded cursor-pointer transition-colors duration-200 w-full max-w-full box-border wrap-break-word hover-bg-panel border-[var(--color-border)] bg-[var(--color-panel-bg)]"
        >
            <RawPreview content=content />
            <div class="text-xs mt-1 text-[var(--color-text-secondary)]">{source}</div>
            <div class="text-xs mt-1 text-[var(--color-text-secondary)]">{created}</div>
        </div>
    }
}

#[component]
fn RawPreview(content: RawContent) -> impl IntoView {
    view! {
        <div class="flex items-center gap-2 mb-2">
            {match &content {
                RawContent::Text(text) => {
                    let preview_text = if text.content.len() > 50 {
                        format!("{}...", &text.content[..50])
                    } else {
                        text.content.clone()
                    };
                    view! {
                        <>
                            <div class="text-xl">"📄"</div>
                            <div class="flex-1 text-sm wrap-break-word text-[var(--color-text-secondary)]">{preview_text}</div>
                        </>
                    }.into_any()
                }
                RawContent::Image(img) => {
                    // Load thumbnail if available
                    let thumbnail_url = RwSignal::new(Option::<String>::None);
                    let thumbnail_url = thumbnail_url.clone();
                    let img = img.clone();

                    if let Some(thumb_blob_id) = img.thumbnail_blob_id {
                        web_sys::console::log_1(
                            &format!("Thumbnail exists for image: {}", thumb_blob_id.0).into(),
                        );
                        spawn_local(async move {
                            match tauri_api::get_blob(thumb_blob_id.0.to_string()).await {
                                Ok(data) => {
                                    // Convert blob data to base64 data URL using web APIs
                                    // Convert Vec<u8> to binary string for btoa
                                    let binary_string: String = data.iter().map(|&b| b as char).collect();
                                    let window = web_sys::window().unwrap();
                                    let base64 = match js_sys::Reflect::get(&window, &wasm_bindgen::JsValue::from_str("btoa")) {
                                        Ok(btoa_fn) => {
                                            match js_sys::Function::from(btoa_fn)
                                                .call1(&wasm_bindgen::JsValue::NULL, &wasm_bindgen::JsValue::from_str(&binary_string))
                                            {
                                                Ok(result) => result.as_string().unwrap_or_default(),
                                                Err(_) => String::new(),
                                            }
                                        }
                                        Err(_) => String::new(),
                                    };

                                    if !base64.is_empty() {
                                        let format = img.format.as_deref().unwrap_or("jpeg");
                                        let mime_type = match format {
                                            "png" => "image/png",
                                            "jpeg" | "jpg" => "image/jpeg",
                                            "gif" => "image/gif",
                                            "webp" => "image/webp",
                                            _ => "image/jpeg",
                                        };
                                        let data_url = format!("data:{};base64,{}", mime_type, base64);
                                        thumbnail_url.set(Some(data_url));
                                    }
                                }
                                Err(_) => {
                                    // Fallback to emoji on error
                                }
                            }
                        });
                    } else {
                        web_sys::console::log_1(
                            &"Thumbnail does not exist for image, using fallback emoji".into(),
                        );
                    }

                    view! {
                        <>
                            <div class="text-xl w-5 h-5 flex items-center justify-center shrink-0">
                                    {move || {
                                        if let Some(url) = thumbnail_url.get() {
                                        view! { <img class="w-5 h-5 object-cover rounded shrink-0" src=url alt="Thumbnail" /> }.into_any()
                                        } else {
                                            view! { <span>"🖼️"</span> }.into_any()
                                        }
                                    }}
                                </div>
                            <div class="flex-1 text-sm wrap-break-word text-[var(--color-text-secondary)]">{format!("Image ({}x{})", img.width, img.height)}</div>
                        </>
                    }.into_any()
                }
            }}
        </div>
    }
}

fn format_raw_source(source: &RawSource) -> String {
    match source {
        | RawSource::Clipboard { application } => {
            format!(
                "Clipboard{}",
                application.as_ref().map(|a| format!(" ({})", a)).unwrap_or_default()
            )
        }
        | RawSource::FileWatcher { original_path } => {
            format!("File: {}", original_path.display())
        }
        | RawSource::ManualImport => "Manual Import".to_string(),
    }
}
