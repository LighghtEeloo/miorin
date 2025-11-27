#![allow(non_snake_case)]

use dioxus::prelude::*;
use miorin_core::prelude::*;
use chrono::Utc;
use uuid::{Uuid, Timestamp, NoContext};

// Include the stylesheet asset
static STYLESHEET: Asset = asset!("/assets/styles.css");

// Helper function to generate UUID v7
fn new_uuid_v7() -> Uuid {
    Uuid::new_v7(Timestamp::now(NoContext))
}

// Helper function to extract plain text from RichText
fn rich_text_to_string(rich_text: &RichText) -> String {
    rich_text.segments.iter()
        .map(|seg| seg.text.as_str())
        .collect::<Vec<_>>()
        .join("")
}

// Helper function to get block kind display name
fn block_kind_name(kind: &BlockKind) -> String {
    match kind {
        BlockKind::Document(_) => "Document",
        BlockKind::Paragraph { .. } => "Paragraph",
        BlockKind::Heading { .. } => "Heading",
        BlockKind::Todo { .. } => "Todo",
        BlockKind::Quote { .. } => "Quote",
        BlockKind::BulletedListItem { .. } => "Bullet",
        BlockKind::NumberedListItem { .. } => "Numbered",
        BlockKind::Image { .. } => "Image",
        BlockKind::RawReference { .. } => "Raw Reference",
    }.to_string()
}

// Helper function to get block title/preview
fn block_title(block: &Block) -> String {
    match &block.kind {
        BlockKind::Document(doc) => doc.inner.title.clone(),
        BlockKind::Paragraph { text } => {
            let txt = rich_text_to_string(text);
            if txt.len() > 50 {
                format!("{}...", &txt[..50])
            } else {
                txt
            }
        },
        BlockKind::Heading { text, .. } => rich_text_to_string(text),
        BlockKind::Todo { text, .. } => rich_text_to_string(text),
        BlockKind::Quote { text } => {
            let txt = rich_text_to_string(text);
            if txt.len() > 50 {
                format!("{}...", &txt[..50])
            } else {
                txt
            }
        },
        BlockKind::BulletedListItem { text } => rich_text_to_string(text),
        BlockKind::NumberedListItem { text, .. } => rich_text_to_string(text),
        BlockKind::Image { caption, .. } => {
            caption.as_ref()
                .map(|c| rich_text_to_string(c))
                .unwrap_or_else(|| "Image".to_string())
        },
        BlockKind::RawReference { .. } => "Raw Reference".to_string(),
    }
}

// Helper function to format raw material preview text
fn raw_preview_text(material: &RawPreview) -> String {
    match &material.kind {
        RawKindPreview::Text { first_chars } => first_chars.clone(),
        RawKindPreview::Image { width, height, .. } => {
            format!("[Image: {}x{}]", width, height)
        }
    }
}

// Helper function to format date
fn format_date(dt: &chrono::DateTime<chrono::Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M").to_string()
}

pub fn App() -> Element {
    // Dummy raw materials using RawPreview
    let raw_materials = use_signal(|| vec![
        RawPreview {
            id: RawId(new_uuid_v7()),
            created_at: Utc::now(),
            kind: RawKindPreview::Text {
                first_chars: "Discussed project timeline and deliverables...".to_string(),
            },
            title: Some("Meeting Notes".to_string()),
            summary: Some("Project planning discussion".to_string()),
        },
        RawPreview {
            id: RawId(new_uuid_v7()),
            created_at: Utc::now(),
            kind: RawKindPreview::Text {
                first_chars: "fn main() { println!(\"Hello, world!\"); }".to_string(),
            },
            title: Some("Code Snippet".to_string()),
            summary: None,
        },
        RawPreview {
            id: RawId(new_uuid_v7()),
            created_at: Utc::now(),
            kind: RawKindPreview::Text {
                first_chars: "Random thought: The best ideas come when you least expect them.".to_string(),
            },
            title: None,
            summary: None,
        },
        RawPreview {
            id: RawId(new_uuid_v7()),
            created_at: Utc::now(),
            kind: RawKindPreview::Image {
                thumbnail_blob_id: None,
                width: 1920,
                height: 1080,
            },
            title: Some("Image".to_string()),
            summary: None,
        },
    ]);

    // Dummy blocks for display
    let now = Utc::now();
    let blocks = use_signal(|| {
        let mut all_blocks = Vec::new();
        
        // Add document blocks
        let doc1 = Document {
            id: BlockId(new_uuid_v7()),
            created_at: now,
            updated_at: now,
            vibe: None,
            inner: DocumentInner {
                title: "Daily Journal - Jan 15".to_string(),
                blocks: vec![],
                links: vec![],
            },
        };
        all_blocks.push(Block {
            id: doc1.id,
            parent: None,
            index: 0,
            kind: BlockKind::Document(doc1),
        });

        let doc2 = Document {
            id: BlockId(new_uuid_v7()),
            created_at: now,
            updated_at: now,
            vibe: None,
            inner: DocumentInner {
                title: "Project Ideas".to_string(),
                blocks: vec![],
                links: vec![],
            },
        };
        all_blocks.push(Block {
            id: doc2.id,
            parent: None,
            index: 0,
            kind: BlockKind::Document(doc2),
        });
        
        // Add some other block types
        let block_id1 = BlockId(new_uuid_v7());
        all_blocks.push(Block {
            id: block_id1,
            parent: None,
            index: 0,
            kind: BlockKind::Paragraph {
                text: RichText {
                    segments: vec![RichTextSegment {
                        text: "Meeting Notes: Discussed the new feature implementation...".to_string(),
                        marks: TextMarks::default(),
                    }],
                },
            },
        });

        let block_id2 = BlockId(new_uuid_v7());
        all_blocks.push(Block {
            id: block_id2,
            parent: None,
            index: 0,
            kind: BlockKind::Todo {
                text: RichText {
                    segments: vec![RichTextSegment {
                        text: "Review PR #42".to_string(),
                        marks: TextMarks::default(),
                    }],
                },
                done: false,
            },
        });

        all_blocks
    });

    rsx! {
        document::Stylesheet { href: STYLESHEET }
        div {
            class: "app-container",
            // Left panel: Blocks List
            div {
                class: "panel left-panel",
                h2 { class: "panel-title", "Blocks" }
                div {
                    class: "panel-content",
                    for block in blocks.read().iter() {
                        div {
                            class: "block-item",
                            onclick: move |_| {
                                // TODO: Handle selection
                            },
                            div {
                                class: "block-header",
                                h3 { class: "block-title", "{block_title(block)}" }
                                span { class: "block-kind", "{block_kind_name(&block.kind)}" }
                            }
                        }
                    }
                }
            }

            // Middle panel: Main Workspace
            div {
                class: "panel middle-panel",
                h2 { class: "panel-title", "Workspace" }
                div {
                    class: "panel-content workspace",
                    p { class: "workspace-placeholder", "Your workspace will appear here" }
                }
            }

            // Right panel: Raw Material List
            div {
                class: "panel right-panel",
                h2 { class: "panel-title", "Raw Material" }
                div {
                    class: "panel-content",
                    for material in raw_materials.read().iter() {
                        div {
                            class: "raw-material-item",
                            onclick: move |_| {
                                // TODO: Handle selection
                            },
                            div {
                                class: "raw-material-header",
                                if let Some(title) = &material.title {
                                    h3 { class: "raw-material-title", "{title}" }
                                }
                                span { 
                                    class: "raw-material-source", 
                                    "{format_date(&material.created_at)}"
                                }
                            }
                            p { 
                                class: "raw-material-preview", 
                                "{raw_preview_text(material)}"
                            }
                            if let Some(summary) = &material.summary {
                                p { class: "raw-material-summary", "{summary}" }
                            }
                        }
                    }
                }
            }
        }
    }
}