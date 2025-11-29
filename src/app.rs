use crate::panel;
use crate::card;
use crate::tauri_api;
use leptos::prelude::*;
use leptos_icons::Icon;
use icondata::{LuPlus, LuSettings};
use miorin_core::prelude::*;
use wasm_bindgen_futures::spawn_local;

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
                    <MainView cubes=cubes raw_entries=raw_entries open_settings=open_settings />
                }.into_any()
            }
        }}
    }
}

#[component]
fn MainView(
    cubes: RwSignal<Vec<Cube>>, raw_entries: RwSignal<Vec<Raw>>,
    open_settings: impl Fn(web_sys::MouseEvent) + 'static,
) -> impl IntoView {
    view! {
        <div
            class="grid h-screen max-h-screen overflow-hidden bg-[var(--color-bg)]"
            style="grid-template-columns: 250px 1fr 300px;"
        >
            <GlacierPanel cubes=cubes />
            <Workspace />
            <StreamPanel raw_entries=raw_entries open_settings=open_settings />
        </div>
    }
}

#[component]
fn GlacierPanel(cubes: RwSignal<Vec<Cube>>) -> impl IntoView {
    let cubes_clone = cubes.clone();
    let create_cube_action = move |_| {
        let cubes_signal = cubes_clone.clone();
        spawn_local(async move {
            // Create a dummy paragraph cube
            let dummy_paragraph = Paragraph {
                text: RichText {
                    segments: vec![RichTextSegment {
                        text: "Dummy cube".to_string(),
                        marks: TextMarks::default(),
                    }],
                },
            };

            match tauri_api::create_cube(true, CubeContent::Paragraph(dummy_paragraph)).await {
                | Ok(_) => {
                    // Reload cubes
                    match tauri_api::get_all_cubes().await {
                        | Ok(cubes_data) => cubes_signal.set(cubes_data),
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
            header_actions=view! {
                <button
                    class="p-1.5 text-white border-none rounded cursor-pointer flex items-center justify-center w-7 h-7 min-w-7 transition-colors duration-200 bg-[var(--color-primary,#007bff)] hover-bg-primary"
                    on:click=create_cube_action
                >
                    <Icon icon=LuPlus width="16" height="16" />
                </button>
            }.into_any()
        >
            <card::CardList items=move || {
                let cubes_data = cubes.get();
                cubes_data.into_iter()
                    .filter(|cube| cube.pin)
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
                class="p-4 border-b border-[var(--color-border)] bg-[var(--color-panel-header-bg)]"
            >
                <h2 class="m-0 text-base font-semibold text-[var(--color-text)]">"Workspace"</h2>
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
) -> impl IntoView {
    view! {
        <panel::Panel
            title="Stream"
            header_actions=view! {
                <button
                    class="p-1.5 text-white border-none rounded cursor-pointer flex items-center justify-center w-7 h-7 min-w-7 transition-colors duration-200 bg-[var(--color-secondary,#6c757d)] hover-bg-secondary"
                    on:click=open_settings
                >
                    <Icon icon=LuSettings width="16" height="16" />
                </button>
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
    let (title, created_at) = match &cube.content {
        | CubeContent::Document(doc) => (doc.inner.title.clone(), doc.created_at),
        | CubeContent::PreOrder(po) => (format!("PreOrder"), po.created_at),
        | CubeContent::Order(order) => (format!("Order"), order.created_at),
        | CubeContent::Paragraph(_) => (format!("Paragraph"), chrono::Utc::now()),
        | CubeContent::Heading(heading) => {
            let text = heading.text.segments.iter().map(|s| s.text.clone()).collect::<String>();
            (
                if text.is_empty() { format!("Heading H{}", heading.level) } else { text },
                chrono::Utc::now(),
            )
        }
        | CubeContent::Todo(todo) => {
            let text = todo.text.segments.iter().map(|s| s.text.clone()).collect::<String>();
            (if text.is_empty() { "Todo".to_string() } else { text }, chrono::Utc::now())
        }
        | CubeContent::Quote(quote) => {
            let text = quote.text.segments.iter().map(|s| s.text.clone()).collect::<String>();
            (if text.is_empty() { "Quote".to_string() } else { text }, chrono::Utc::now())
        }
        | CubeContent::Image(_) => (format!("Image"), chrono::Utc::now()),
        | CubeContent::RawReference(_) => (format!("Raw Reference"), chrono::Utc::now()),
    };
    let created = created_at.format("%Y-%m-%d %H:%M").to_string();
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
