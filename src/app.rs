use crate::panel;
use crate::card;
use crate::tauri_api;
use leptos::prelude::*;
use leptos_icons::Icon;
use icondata::{LuPlus, LuSettings};
use miorin_core::prelude::*;
use styled::style;
use wasm_bindgen_futures::spawn_local;

#[component]
fn MainView(
    cubes: RwSignal<Vec<Cube>>, raw_entries: RwSignal<Vec<Raw>>,
    open_settings: impl Fn(web_sys::MouseEvent) + 'static,
) -> impl IntoView {
    let app_container_styles = style! {
        .app-container {
            display: grid;
            grid-template-columns: 250px 1fr 300px;
            height: 100vh;
            max-height: 100vh;
            overflow: hidden;
            background-color: var(--color-bg);
        }
    };

    styled::view! { app_container_styles,
        <div class="app-container">
            <GlacierPanel cubes=cubes />
            <Workspace />
            <StreamPanel raw_entries=raw_entries open_settings=open_settings />
        </div>
    }
}

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
                            web_sys::console::error_1(&format!("Failed to reload cubes: {}", e).into());
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
                    style="padding: 6px; background: var(--color-primary, #007bff); color: white; border: none; border-radius: 4px; cursor: pointer; display: flex; align-items: center; justify-content: center; width: 28px; height: 28px; min-width: 28px; transition: background-color 0.2s;"
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
    let workspace_styles = style! {
        .workspace {
            display: flex;
            flex-direction: column;
            height: 100%;
            border-right: 1px solid var(--color-border);
            background-color: var(--color-workspace-bg);
            overflow: hidden;
        }
    };

    let workspace_header_styles = style! {
        .workspace-header {
            padding: 1rem;
            border-bottom: 1px solid var(--color-border);
            background-color: var(--color-panel-header-bg);
        }
        .workspace-header h2 {
            margin: 0;
            font-size: 1rem;
            font-weight: 600;
            color: var(--color-text);
        }
    };

    let workspace_content_styles = style! {
        .workspace-content {
            flex: 1;
            overflow-y: auto;
            padding: 0.5rem;
        }
    };

    let workspace_editor_styles = style! {
        .workspace-editor {
            padding: 2rem;
            min-height: 100%;
        }
    };

    let placeholder_text_styles = style! {
        .placeholder-text {
            color: var(--color-text-muted);
            font-style: italic;
            text-align: center;
            margin-top: 3rem;
        }
    };

    styled::view! { workspace_styles,
        <div class="workspace">
            {styled::view! { workspace_header_styles,
                <div class="workspace-header">
                    <h2>"Workspace"</h2>
                </div>
            }}
            {styled::view! { workspace_content_styles,
                <div class="workspace-content">
                    {styled::view! { workspace_editor_styles,
                        <div class="workspace-editor">
                            {styled::view! { placeholder_text_styles,
                                <p class="placeholder-text">"Select a cube from Glacier or drag raw material from Stream to start editing..."</p>
                            }}
                        </div>
                    }}
                </div>
            }}
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
            wrapper=true
            header_actions=view! {
                <button class="settings-button" on:click=open_settings>
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
    let entry_item_styles = style! {
        .entry-item {
            padding: 0.75rem;
            border: 1px solid var(--color-border);
            border-radius: 4px;
            background-color: var(--color-panel-bg);
            cursor: pointer;
            transition: background-color 0.2s;
        }
        .entry-item:hover {
            background-color: var(--color-hover);
        }
    };

    let entry_title_styles = style! {
        .entry-title {
            font-weight: 500;
            color: var(--color-text);
            margin-bottom: 0.25rem;
        }
    };

    let entry_meta_styles = style! {
        .entry-meta {
            font-size: 0.75rem;
            color: var(--color-text-secondary);
            margin-top: 0.25rem;
        }
    };

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
    styled::view! { entry_item_styles,
        <div class="entry-item">
            {styled::view! { entry_title_styles,
                <div class="entry-title">{title}</div>
            }}
            {styled::view! { entry_meta_styles,
                <div class="entry-meta">{created}</div>
            }}
        </div>
    }
}

#[component]
fn RawEntry(raw: Raw) -> impl IntoView {
    let entry_item_styles = style! {
        .entry-item {
            padding: 0.75rem;
            border: 1px solid var(--color-border);
            border-radius: 4px;
            background-color: var(--color-panel-bg);
            cursor: pointer;
            transition: background-color 0.2s;
        }
        .entry-item:hover {
            background-color: var(--color-hover);
        }
    };

    let entry_meta_styles = style! {
        .entry-meta {
            font-size: 0.75rem;
            color: var(--color-text-secondary);
            margin-top: 0.25rem;
        }
    };

    let source = format_raw_source(&raw.inner.source);
    let created = raw.created_at.format("%Y-%m-%d %H:%M").to_string();
    let kind = raw.inner.content.clone();
    styled::view! { entry_item_styles,
        <div class="entry-item raw-entry">
            <RawPreview kind=kind />
            {{
                let entry_meta_styles2 = style! {
                    .entry-meta {
                        font-size: 0.75rem;
                        color: var(--color-text-secondary);
                        margin-top: 0.25rem;
                    }
                };
                styled::view! { entry_meta_styles2,
                    <div class="entry-meta">{source}</div>
                }
            }}
            {styled::view! { entry_meta_styles,
                <div class="entry-meta">{created}</div>
            }}
        </div>
    }
}

#[component]
fn RawPreview(kind: RawContent) -> impl IntoView {
    let raw_preview_styles = style! {
        .raw-preview {
            display: flex;
            align-items: center;
            gap: 0.5rem;
            margin-bottom: 0.5rem;
        }
    };

    styled::view! { raw_preview_styles,
        <div class="raw-preview">
            {match &kind {
                RawContent::Text(text) => {
                    let preview_text = if text.content.len() > 50 {
                        format!("{}...", &text.content[..50])
                    } else {
                        text.content.clone()
                    };
                    let raw_icon_styles2 = style! {
                        .raw-icon {
                            font-size: 1.25rem;
                        }
                    };
                    let raw_content_styles2 = style! {
                        .raw-content {
                            flex: 1;
                            font-size: 0.875rem;
                            color: var(--color-text-secondary);
                            word-break: break-word;
                        }
                    };
                    view! {
                        <>
                            {styled::view! { raw_icon_styles2,
                                <div class="raw-icon">"📄"</div>
                            }}
                            {styled::view! { raw_content_styles2,
                                <div class="raw-content">{preview_text}</div>
                            }}
                        </>
                    }
                }
                RawContent::Image(img) => {
                    let raw_icon_styles3 = style! {
                        .raw-icon {
                            font-size: 1.25rem;
                        }
                    };
                    let raw_content_styles3 = style! {
                        .raw-content {
                            flex: 1;
                            font-size: 0.875rem;
                            color: var(--color-text-secondary);
                            word-break: break-word;
                        }
                    };
                    view! {
                        <>
                            {styled::view! { raw_icon_styles3,
                                <div class="raw-icon">"🖼️"</div>
                            }}
                            {styled::view! { raw_content_styles3,
                                <div class="raw-content">{format!("Image ({}x{})", img.width, img.height)}</div>
                            }}
                        </>
                    }
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
