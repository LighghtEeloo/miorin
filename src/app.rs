use crate::tauri_api;
use leptos::prelude::*;
use miorin_core::prelude::*;
use styled::style;
use wasm_bindgen_futures::spawn_local;

#[component]
pub fn App() -> impl IntoView {
    let app_container_styles = style! {
        .app-container {
            display: grid;
            grid-template-columns: 250px 1fr 300px;
            height: 100vh;
            overflow: hidden;
            background-color: var(--color-bg);
        }
    };

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
                if entries.is_empty() {
                    web_sys::console::log_1(
                        &"Database is empty, creating dummy raw entries...".into(),
                    );
                    // Seed with dummy data if database is empty
                    let dummy_entries = create_dummy_raw_entries();
                    let dummy_count = dummy_entries.len();
                    web_sys::console::log_1(
                        &format!("Created {} dummy entry definitions", dummy_count).into(),
                    );
                    let mut created_entries = Vec::new();
                    for (index, inner) in dummy_entries.into_iter().enumerate() {
                        web_sys::console::log_1(
                            &format!("Creating dummy entry {} of {}", index + 1, dummy_count)
                                .into(),
                        );
                        match tauri_api::create_raw_entry(inner).await {
                            | Ok(raw) => {
                                web_sys::console::log_1(
                                    &format!(
                                        "Successfully created raw entry with id: {}",
                                        raw.id.0
                                    )
                                    .into(),
                                );
                                created_entries.push(raw);
                            }
                            | Err(e) => {
                                web_sys::console::error_1(
                                    &format!(
                                        "Failed to create dummy raw entry {}: {}",
                                        index + 1,
                                        e
                                    )
                                    .into(),
                                );
                            }
                        }
                    }
                    web_sys::console::log_1(
                        &format!("Setting {} created entries to signal", created_entries.len())
                            .into(),
                    );
                    raw_entries_clone.set(created_entries);
                } else {
                    web_sys::console::log_1(
                        &format!("Database has {} entries, using existing data", entries.len())
                            .into(),
                    );
                    raw_entries_clone.set(entries);
                }
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

    styled::view! { app_container_styles,
        <div class="app-container">
            <GlacierPanel cubes=cubes />
            <WorkspacePanel />
            <StreamPanel raw_entries=raw_entries />
        </div>
    }
}

#[component]
fn GlacierPanel(cubes: RwSignal<Vec<Cube>>) -> impl IntoView {
    let panel_styles = style! {
        .panel {
            display: flex;
            flex-direction: column;
            height: 100%;
            border-right: 1px solid var(--color-border);
            background-color: var(--color-panel-bg);
            overflow: hidden;
        }
    };

    let panel_header_styles = style! {
        .panel-header {
            padding: 1rem;
            border-bottom: 1px solid var(--color-border);
            background-color: var(--color-panel-header-bg);
        }
        .panel-header h2 {
            margin: 0;
            font-size: 1rem;
            font-weight: 600;
            color: var(--color-text);
        }
    };

    let panel_content_styles = style! {
        .panel-content {
            flex: 1;
            overflow-y: auto;
            padding: 0.5rem;
        }
    };

    let entries_list_styles = style! {
        .entries-list {
            display: flex;
            flex-direction: column;
            gap: 0.5rem;
        }
    };

    styled::view! { panel_styles,
        <div class="panel">
            {styled::view! { panel_header_styles,
                <div class="panel-header">
                    <h2>"Glacier"</h2>
                </div>
            }}
            {styled::view! { panel_content_styles,
                <div class="panel-content">
                    {styled::view! { entries_list_styles,
                        <div class="entries-list">
                            {move || {
                                let cubes_data = cubes.get();
                                cubes_data.into_iter()
                                    .filter(|cube| cube.pin)
                                    .map(|cube| view! { <CubeEntry cube=cube /> })
                                    .collect::<Vec<_>>()
                            }}
                        </div>
                    }}
                </div>
            }}
        </div>
    }
}

#[component]
fn WorkspacePanel() -> impl IntoView {
    let workspace_panel_combined = style! {
        .panel {
            display: flex;
            flex-direction: column;
            height: 100%;
            border-right: 1px solid var(--color-border);
            background-color: var(--color-panel-bg);
            overflow: hidden;
        }
        .workspace-panel {
            background-color: var(--color-workspace-bg);
        }
    };

    let panel_header_styles = style! {
        .panel-header {
            padding: 1rem;
            border-bottom: 1px solid var(--color-border);
            background-color: var(--color-panel-header-bg);
        }
        .panel-header h2 {
            margin: 0;
            font-size: 1rem;
            font-weight: 600;
            color: var(--color-text);
        }
    };

    let panel_content_styles = style! {
        .panel-content {
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

    styled::view! { workspace_panel_combined,
        <div class="panel workspace-panel">
            {styled::view! { panel_header_styles,
                <div class="panel-header">
                    <h2>"Workspace"</h2>
                </div>
            }}
            {styled::view! { panel_content_styles,
                <div class="panel-content">
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
fn StreamPanel(raw_entries: RwSignal<Vec<Raw>>) -> impl IntoView {
    let panel_styles = style! {
        .panel {
            display: flex;
            flex-direction: column;
            height: 100%;
            border-right: 1px solid var(--color-border);
            background-color: var(--color-panel-bg);
            overflow: hidden;
        }
        .panel:last-child {
            border-right: none;
        }
    };

    let panel_header_styles = style! {
        .panel-header {
            padding: 1rem;
            border-bottom: 1px solid var(--color-border);
            background-color: var(--color-panel-header-bg);
        }
        .panel-header h2 {
            margin: 0;
            font-size: 1rem;
            font-weight: 600;
            color: var(--color-text);
        }
    };

    let panel_content_styles = style! {
        .panel-content {
            flex: 1;
            overflow-y: auto;
            padding: 0.5rem;
        }
    };

    let entries_list_styles = style! {
        .entries-list {
            display: flex;
            flex-direction: column;
            gap: 0.5rem;
        }
    };

    styled::view! { panel_styles,
        <div class="panel">
            {styled::view! { panel_header_styles,
                <div class="panel-header">
                    <h2>"Stream"</h2>
                </div>
            }}
            {styled::view! { panel_content_styles,
                <div class="panel-content">
                    {styled::view! { entries_list_styles,
                        <div class="entries-list">
                            {move || {
                                let entries = raw_entries.get();
                                entries.into_iter()
                                    .map(|raw| view! { <RawEntry raw=raw /> })
                                    .collect::<Vec<_>>()
                            }}
                        </div>
                    }}
                </div>
            }}
        </div>
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

/// Create dummy raw entries for seeding the database
fn create_dummy_raw_entries() -> Vec<RawInner> {
    use uuid::Uuid;

    vec![
        RawInner {
            source: RawSource::Clipboard {
                application: Some("Chrome".to_string()),
            },
            content: RawContent::Text(TextRaw {
                content: "This is a note I copied from a website about Rust programming.".to_string(),
                mime_type: Some("text/plain".to_string()),
                language: Some("en".to_string()),
            }),
        },
        RawInner {
            source: RawSource::FileWatcher {
                original_path: std::path::PathBuf::from("/Users/me/Documents/notes.txt"),
            },
            content: RawContent::Text(TextRaw {
                content: "A longer piece of text that was automatically collected from a watched folder. This demonstrates how the file watcher works.".to_string(),
                mime_type: Some("text/plain".to_string()),
                language: Some("en".to_string()),
            }),
        },
        RawInner {
            source: RawSource::Clipboard {
                application: Some("VSCode".to_string()),
            },
            content: RawContent::Text(TextRaw {
                content: "function hello() { console.log('Hello, world!'); }".to_string(),
                mime_type: Some("text/plain".to_string()),
                language: Some("javascript".to_string()),
            }),
        },
        RawInner {
            source: RawSource::ManualImport,
            content: RawContent::Image(ImageRaw {
                blob_id: BlobId(Uuid::now_v7()),
                width: 1920,
                height: 1080,
                format: Some("png".to_string()),
                thumbnail_blob_id: None,
                dominant_color_rgb: Some([120, 150, 200]),
            }),
        },
    ]
}
