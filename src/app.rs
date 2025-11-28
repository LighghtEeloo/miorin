use leptos::prelude::*;
use miorin_core::prelude::*;
use chrono::Utc;
use uuid::Uuid;

#[component]
pub fn App() -> impl IntoView {
    // Create dummy raw entries
    let raw_entries = create_dummy_raw_entries();
    
    // Create dummy documents
    let documents = create_dummy_documents();

    view! {
        <div class="app-container">
            <div class="panel glacier-panel">
                <div class="panel-header">
                    <h2>"Glacier"</h2>
                </div>
                <div class="panel-content">
                    <div class="entries-list">
                        {documents.into_iter().map(|doc| {
                            let title = doc.inner.title.clone();
                            let created = doc.created_at.format("%Y-%m-%d %H:%M").to_string();
                            view! {
                                <div class="entry-item">
                                    <div class="entry-title">{title}</div>
                                    <div class="entry-meta">{created}</div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
            </div>

            <div class="panel workspace-panel">
                <div class="panel-header">
                    <h2>"Workspace"</h2>
                </div>
                <div class="panel-content">
                    <div class="workspace-editor">
                        <p class="placeholder-text">"Select a cube from Glacier or drag raw material from Stream to start editing..."</p>
                    </div>
                </div>
            </div>

            <div class="panel stream-panel">
                <div class="panel-header">
                    <h2>"Stream"</h2>
                </div>
                <div class="panel-content">
                    <div class="entries-list">
                        {raw_entries.into_iter().map(|raw| {
                            let preview = match &raw.inner.kind {
                                RawKind::Text(text) => {
                                    let preview_text = if text.content.len() > 50 {
                                        format!("{}...", &text.content[..50])
                                    } else {
                                        text.content.clone()
                                    };
                                    view! {
                                        <div class="raw-preview">
                                            <div class="raw-icon">"📄"</div>
                                            <div class="raw-content">{preview_text}</div>
                                        </div>
                                    }
                                }
                                RawKind::Image(img) => {
                                    view! {
                                        <div class="raw-preview">
                                            <div class="raw-icon">"🖼️"</div>
                                            <div class="raw-content">{format!("Image ({}x{})", img.width, img.height)}</div>
                                        </div>
                                    }
                                }
                            };
                            let source = match &raw.inner.source {
                                RawSource::Clipboard { application } => {
                                    format!("Clipboard{}", application.as_ref().map(|a| format!(" ({})", a)).unwrap_or_default())
                                }
                                RawSource::FileWatcher { original_path } => {
                                    format!("File: {}", original_path.display())
                                }
                                RawSource::ManualImport => "Manual Import".to_string(),
                            };
                            let created = raw.created_at.format("%Y-%m-%d %H:%M").to_string();
                            view! {
                                <div class="entry-item raw-entry">
                                    {preview}
                                    <div class="entry-meta">{source}</div>
                                    <div class="entry-meta">{created}</div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
            </div>
        </div>
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
                kind: RawKind::Text(TextRaw {
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
                kind: RawKind::Text(TextRaw {
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
                kind: RawKind::Text(TextRaw {
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
                kind: RawKind::Image(ImageRaw {
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
