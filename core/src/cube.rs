use super::*;
use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CubeId(pub Uuid);

pub type Document = Meta<CubeId, DocumentInner>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentInner {
    pub title: String,
    /// Flat list of cubes; use parent/index for structure.
    pub cubes: Vec<Cube>,

    /// Graph edges between cubes.
    pub links: Vec<Link>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cube {
    pub id: CubeId,
    /// None = top-level.
    pub parent: Option<CubeId>,
    /// Order among siblings.
    pub index: u32,
    /// The type of cube.
    pub kind: CubeKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CubeKind {
    Document(Document),
    Paragraph {
        text: RichText,
    },
    Heading {
        level: u8,
        text: RichText,
    },
    Todo {
        text: RichText,
        done: bool,
    },
    Quote {
        text: RichText,
    },
    BulletedListItem {
        text: RichText,
    },
    NumberedListItem {
        text: RichText,
        number: Option<u32>,
    },

    Image {
        blob_id: BlobId,
        caption: Option<RichText>,
        original_raw: Option<RawId>, // provenance
    },

    RawReference {
        raw_id: RawId, // show as an embedded “card”
    },

    // You can grow this over time:
    // Callout {
    //     text: RichText,
    // },
    // Code {
    //     language: Option<String>,
    //     code: String,
    // },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RichText {
    pub segments: Vec<RichTextSegment>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RichTextSegment {
    pub text: String,
    pub marks: TextMarks,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TextMarks {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub code: bool,
    pub strike: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinkId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LinkKind {
    Related,
    Supports,
    Contradicts,
    Explains,
    DerivedFrom,
    Custom(String),
}

pub type Link = Meta<LinkId, LinkInner>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinkInner {
    pub from: CubeId,
    pub to: CubeId,
    pub kind: LinkKind,
}
