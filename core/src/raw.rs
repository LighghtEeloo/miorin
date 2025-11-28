//! Raw material types.

use super::*;
use crate::prelude::*;

/// BlobId is a unique identifier for a binary blob (images, maybe large text).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlobId(pub Uuid);

/// RawId is a unique identifier for a raw entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RawId(pub Uuid);

pub type RawEntry = Meta<RawId, RawEntryInner>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawEntryInner {
    pub source: RawSource,
    pub content: RawContent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawSource {
    Clipboard {
        /// The application that originated the clipboard content,
        /// e.g. "Chrome", "VSCode".
        application: Option<String>,
    },
    FileWatcher {
        original_path: PathBuf,
    },
    ManualImport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, From)]
pub enum RawContent {
    Text(TextRaw),
    Image(ImageRaw),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextRaw {
    pub content: String,
    /// MIME type, e.g. "text/plain", "text/html", etc.
    pub mime_type: Option<String>,
    /// Language, e.g. "en", "ja", etc.
    pub language: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageRaw {
    /// Points into your blob store
    pub blob_id: BlobId,
    pub width: u32,
    pub height: u32,
    /// Format, e.g. "png", "jpeg", etc.
    pub format: Option<String>,
    /// Thumbnails for fast list rendering.
    pub thumbnail_blob_id: Option<BlobId>,
    /// Optional UI sugar for color-coding.
    pub dominant_color_rgb: Option<[u8; 3]>,
}
