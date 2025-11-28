use chrono::Utc;
use leptos::prelude::*;
use miorin_core::prelude::*;
use styled::style;
use uuid::Uuid;

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

    // Create dummy raw entries
    let raw_entries = create_dummy_raw_entries();

    // Create dummy documents
    let documents = create_dummy_documents();

    styled::view! { app_container_styles,
        <div class="app-container">
            <GlacierPanel documents=documents />
            <WorkspacePanel />
            <StreamPanel raw_entries=raw_entries />
        </div>
    }
}

#[component]
fn GlacierPanel(documents: Vec<Document>) -> impl IntoView {
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
                            {documents.into_iter().map(|doc| {
                                view! { <DocumentEntry document=doc /> }
                            }).collect::<Vec<_>>()}
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
fn StreamPanel(raw_entries: Vec<RawEntry>) -> impl IntoView {
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
                            {raw_entries.into_iter().map(|raw| {
                                view! { <RawEntry raw=raw /> }
                            }).collect::<Vec<_>>()}
                        </div>
                    }}
                </div>
            }}
        </div>
    }
}

#[component]
fn DocumentEntry(document: Document) -> impl IntoView {
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

    let title = document.inner.title.clone();
    let created = document.created_at.format("%Y-%m-%d %H:%M").to_string();
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
fn RawEntry(raw: RawEntry) -> impl IntoView {
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
        RawSource::Clipboard { application } => {
            format!(
                "Clipboard{}",
                application
                    .as_ref()
                    .map(|a| format!(" ({})", a))
                    .unwrap_or_default()
            )
        }
        RawSource::FileWatcher { original_path } => {
            format!("File: {}", original_path.display())
        }
        RawSource::ManualImport => "Manual Import".to_string(),
    }
}

fn create_dummy_raw_entries() -> Vec<RawEntry> {
    vec![
        RawEntry {
            id: RawId(Uuid::now_v7()),
            created_at: Utc::now() - chrono::Duration::hours(2),
            updated_at: Utc::now() - chrono::Duration::hours(2),
            vibe: None,
            inner: RawEntryInner {
                source: RawSource::Clipboard {
                    application: Some("Chrome".to_string()),
                },
                content: RawContent::Text(TextRaw {
                    content: "This is a note I copied from a website about Rust programming.".to_string(),
                    mime_type: Some("text/plain".to_string()),
                    language: Some("en".to_string()),
                }),
            },
        },
        RawEntry {
            id: RawId(Uuid::now_v7()),
            created_at: Utc::now() - chrono::Duration::hours(1),
            updated_at: Utc::now() - chrono::Duration::hours(1),
            vibe: None,
            inner: RawEntryInner {
                source: RawSource::FileWatcher {
                    original_path: std::path::PathBuf::from("/Users/me/Documents/notes.txt"),
                },
                content: RawContent::Text(TextRaw {
                    content: "A longer piece of text that was automatically collected from a watched folder. This demonstrates how the file watcher works.".to_string(),
                    mime_type: Some("text/plain".to_string()),
                    language: Some("en".to_string()),
                }),
            },
        },
        RawEntry {
            id: RawId(Uuid::now_v7()),
            created_at: Utc::now() - chrono::Duration::minutes(30),
            updated_at: Utc::now() - chrono::Duration::minutes(30),
            vibe: None,
            inner: RawEntryInner {
                source: RawSource::Clipboard {
                    application: Some("VSCode".to_string()),
                },
                content: RawContent::Text(TextRaw {
                    content: "function hello() { console.log('Hello, world!'); }".to_string(),
                    mime_type: Some("text/plain".to_string()),
                    language: Some("javascript".to_string()),
                }),
            },
        },
        RawEntry {
            id: RawId(Uuid::now_v7()),
            created_at: Utc::now() - chrono::Duration::minutes(15),
            updated_at: Utc::now() - chrono::Duration::minutes(15),
            vibe: None,
            inner: RawEntryInner {
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
        },
    ]
}

fn create_dummy_documents() -> Vec<Document> {
    vec![
        Document {
            id: CubeId(Uuid::now_v7()),
            created_at: Utc::now() - chrono::Duration::days(2),
            updated_at: Utc::now() - chrono::Duration::hours(5),
            vibe: Some(Vibe {
                title: Some("Daily Reflection".to_string()),
                summary: Some("Thoughts about the day's work and learnings".to_string()),
                importance: Some(0.7),
                keywords: vec!["reflection".to_string(), "work".to_string()],
            }),
            inner: DocumentInner {
                title: "Daily Reflection - Nov 26".to_string(),
                cubes: vec![],
                links: vec![],
            },
        },
        Document {
            id: CubeId(Uuid::now_v7()),
            created_at: Utc::now() - chrono::Duration::days(1),
            updated_at: Utc::now() - chrono::Duration::hours(2),
            vibe: None,
            inner: DocumentInner {
                title: "Project Ideas".to_string(),
                cubes: vec![],
                links: vec![],
            },
        },
        Document {
            id: CubeId(Uuid::now_v7()),
            created_at: Utc::now() - chrono::Duration::hours(12),
            updated_at: Utc::now() - chrono::Duration::minutes(30),
            vibe: Some(Vibe {
                title: Some("Meeting Notes".to_string()),
                summary: Some("Discussion about the new feature implementation".to_string()),
                importance: Some(0.9),
                keywords: vec!["meeting".to_string(), "feature".to_string()],
            }),
            inner: DocumentInner {
                title: "Team Meeting - Nov 27".to_string(),
                cubes: vec![],
                links: vec![],
            },
        },
    ]
}
