use super::*;
use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawPreview {
    pub id: RawId,
    pub created_at: DateTime<Utc>,
    pub kind: RawKindPreview,
    pub title: Option<String>,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RawKindPreview {
    Text {
        first_chars: String,
    },
    Image {
        thumbnail_blob_id: Option<BlobId>,
        width: u32,
        height: u32,
    },
}
