use super::*;
use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CubeId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cube {
    pub id: CubeId,
    /// Whether the cube is always shown in the short list.
    pub pin: bool,
    /// The content of the cube.
    pub content: CubeContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CubeContent {
    Document(Document),
    PreOrder(PreOrder),
    Order(Order),
    Paragraph(Paragraph),
    Heading(Heading),
    Todo(Todo),
    Quote(Quote),
    Image(Image),
    RawReference(RawReference),
    // You can grow this over time:
    // Callout {
    //     text: RichText,
    // },
    // Code {
    //     language: Option<String>,
    //     code: String,
    // },
}

pub type Document = Meta<CubeId, DocumentInner>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentInner {
    pub title: String,
    /// Flat list of cubes; use parent/index for structure.
    pub cubes: Vec<Cube>,

    /// Graph edges between cubes.
    pub links: Vec<Link>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PreOrderId(pub Uuid);

pub type PreOrder = Meta<PreOrderId, PreOrderInner>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreOrderEdge {
    pub from: CubeId,
    pub to: CubeId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreOrderInner {
    pub dag: daggy::Dag<CubeId, PreOrderEdge>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OrderId(pub Uuid);

pub type Order = Meta<OrderId, OrderInner>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderStyle {
    /// Bulleted list, e.g. *, **, ***, ...
    Bullet,
    /// One-indexed, e.g. 1, 2, 3, ...
    OneIndexed,
    /// Zero-indexed, e.g. 0, 1, 2, ...
    ZeroIndexed,
    /// Latin, e.g. a, b, c, ...
    Latin,
    /// Greek, e.g. α, β, γ, ...
    Greek,
    /// Roman, e.g. I, II, III, ...
    Roman,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderInner {
    pub style: OrderStyle,
    pub items: Vec<CubeId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Paragraph {
    pub text: RichText,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Heading {
    pub level: u8,
    pub text: RichText,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Todo {
    pub text: RichText,
    pub done: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Quote {
    pub text: RichText,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Image {
    pub blob_id: BlobId,
    pub caption: Option<RichText>,
    pub original_raw: Option<RawId>, // provenance
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawReference {
    pub raw_id: RawId, // show as an embedded "card"
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
