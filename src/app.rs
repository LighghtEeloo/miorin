use crate::panel;
use crate::card;
use crate::tauri_api;
use crate::button;
use crate::color::ColorVariant;
use leptos::prelude::*;
use leptos_icons::Icon;
use icondata::{LuPlus, LuSettings};
use miorin_core::prelude::*;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen::JsCast;
use std::rc::Rc;
use std::cell::RefCell;

#[component]
pub fn App() -> impl IntoView {
    // Load raw entries from database
    let raw_entries = RwSignal::new(Vec::<Raw>::new());
    let raw_entries_clone = raw_entries.clone();
    spawn_local(async move {
        web_sys::console::log_1(&"Starting to load raw entries from database...".into());
        match tauri_api::get_all_raw_entries().await {
            | Ok(entries) => {
                web_sys::console::log_1(
                    &format!("Successfully loaded {} raw entries", entries.len()).into(),
                );
                raw_entries_clone.set(entries);
            }
            | Err(e) => {
                web_sys::console::error_1(&format!("Failed to load raw entries: {}", e).into());
            }
        }
        web_sys::console::log_1(&"Finished loading raw entries".into());
    });

    // Load cubes from database
    let cubes = RwSignal::new(Vec::<Cube>::new());
    let cubes_clone = cubes.clone();
    spawn_local(async move {
        match tauri_api::get_all_cubes().await {
            | Ok(cubes_data) => cubes_clone.set(cubes_data),
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
    let left_width_clone = left_width.clone();
    let right_width_clone = right_width.clone();
    spawn_local(async move {
        match tauri_api::get_panel_left_width().await {
            | Ok(width) => left_width_clone.set(width),
            | Err(e) => {
                web_sys::console::warn_1(&format!("Failed to load left panel width: {}", e).into());
            }
        }
        match tauri_api::get_panel_right_width().await {
            | Ok(width) => right_width_clone.set(width),
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
    cubes: RwSignal<Vec<Cube>>, 
    raw_entries: RwSignal<Vec<Raw>>,
    open_settings: impl Fn(web_sys::MouseEvent) + 'static,
    left_width: RwSignal<f64>,
    right_width: RwSignal<f64>,
) -> impl IntoView {

    // Debounce save timers
    let left_save_timeout = Rc::new(RefCell::new(None::<i32>));
    let right_save_timeout = Rc::new(RefCell::new(None::<i32>));

    let on_resize_left = {
        let left_width = left_width.clone();
        let save_timeout = left_save_timeout.clone();
        std::rc::Rc::new(move |new_width: f64| {
            left_width.set(new_width);
            // Debounce saves - clear existing timeout and set a new one
            if let Some(timeout_id) = save_timeout.borrow_mut().take() {
                let window = web_sys::window().unwrap();
                window.clear_timeout_with_handle(timeout_id);
            }
            let save_timeout_clone = save_timeout.clone();
            let left_width_for_save = left_width.clone();
            let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                let width_to_save = left_width_for_save.get();
                spawn_local(async move {
                    if let Err(e) = tauri_api::set_panel_left_width(width_to_save).await {
                        web_sys::console::error_1(
                            &format!("Failed to save left panel width: {}", e).into(),
                        );
                    }
                });
                save_timeout_clone.borrow_mut().take();
            }) as Box<dyn FnMut()>);
            let timeout_id = web_sys::window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    500,
                )
                .unwrap();
            closure.forget();
            *save_timeout.borrow_mut() = Some(timeout_id);
        }) as std::rc::Rc<dyn Fn(f64)>
    };

    let on_resize_right = {
        let right_width = right_width.clone();
        let save_timeout = right_save_timeout.clone();
        std::rc::Rc::new(move |new_width: f64| {
            right_width.set(new_width);
            // Debounce saves - clear existing timeout and set a new one
            if let Some(timeout_id) = save_timeout.borrow_mut().take() {
                let window = web_sys::window().unwrap();
                window.clear_timeout_with_handle(timeout_id);
            }
            let save_timeout_clone = save_timeout.clone();
            let right_width_for_save = right_width.clone();
            let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                let width_to_save = right_width_for_save.get();
                spawn_local(async move {
                    if let Err(e) = tauri_api::set_panel_right_width(width_to_save).await {
                        web_sys::console::error_1(
                            &format!("Failed to save right panel width: {}", e).into(),
                        );
                    }
                });
                save_timeout_clone.borrow_mut().take();
            }) as Box<dyn FnMut()>);
            let timeout_id = web_sys::window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    500,
                )
                .unwrap();
            closure.forget();
            *save_timeout.borrow_mut() = Some(timeout_id);
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
    let cubes_clone = cubes.clone();
    let create_cube_action = move |_| {
        let cubes_signal = cubes_clone.clone();
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

    view! {
        <panel::Panel
            title="Glacier"
            resizable_right=true
            on_resize_right=on_resize_right.clone()
            header_actions=view! {
                <button::ActionButton
                    icon=view! { <Icon icon=LuPlus width="12" height="12" /> }
                    color_variant=ColorVariant::Primary
                    on_click=create_cube_action
                />
            }.into_any()
        >
            <card::CardList items=move || {
                let cubes_data = cubes.get();
                web_sys::console::log_1(
                    &format!("GlacierPanel: Got {} total cubes", cubes_data.len()).into(),
                );
                let pinned: Vec<_> = cubes_data
                    .into_iter()
                    .filter(|cube| {
                        let is_pinned = cube.inner.pin;
                        if !is_pinned {
                            web_sys::console::log_1(
                                &format!("Filtering out cube with pin=false, id={}", cube.id.0).into(),
                            );
                        }
                        is_pinned
                    })
                    .collect();
                web_sys::console::log_1(
                    &format!("GlacierPanel: {} pinned cubes after filter", pinned.len()).into(),
                );
                pinned
                    .into_iter()
                    .map(|cube| view! { <CubeEntry cube=cube /> }.into_any())
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
fn CubeEntry(cube: Cube) -> impl IntoView {
    let title = match &cube.inner.content {
        | CubeContent::Graph(_) => "Graph".to_string(),
        | CubeContent::PreOrder(_) => "PreOrder".to_string(),
        | CubeContent::Order(_) => "Order".to_string(),
        | CubeContent::Point(_) => "Point".to_string(),
        | CubeContent::Meta => "Meta".to_string(),
    };
    let created = cube.created_at.format("%Y-%m-%d %H:%M").to_string();
    view! {
        <div
            class="p-3 border rounded cursor-pointer transition-colors duration-200 hover-bg-panel border-[var(--color-border)] bg-[var(--color-panel-bg)]"
        >
            <div class="font-medium mb-1 text-[var(--color-text)]">{title}</div>
            <div class="text-xs mt-1 text-[var(--color-text-secondary)]">{created}</div>
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
                    let thumbnail_url_clone = thumbnail_url.clone();
                    let img_clone = img.clone();

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
                                        let format = img_clone.format.as_deref().unwrap_or("jpeg");
                                        let mime_type = match format {
                                            "png" => "image/png",
                                            "jpeg" | "jpg" => "image/jpeg",
                                            "gif" => "image/gif",
                                            "webp" => "image/webp",
                                            _ => "image/jpeg",
                                        };
                                        let data_url = format!("data:{};base64,{}", mime_type, base64);
                                        thumbnail_url_clone.set(Some(data_url));
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
