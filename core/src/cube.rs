use super::*;
use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CubeId(pub Uuid);

pub type Cube = Meta<CubeId, CubeInner>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CubeInner {
    /// Whether the cube is always shown in the short list.
    pub pin: bool,
    /// The content of the cube.
    pub content: CubeContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CubeContent {
    Graph(Graph),
    PreOrder(PreOrder),
    Order(Order),
    Point(Point),
    Meta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graph {
    /// Flat list of cubes; use parent/index for structure.
    pub cubes: Vec<Cube>,
    /// Graph edges between cubes.
    pub links: Vec<Link>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreOrderEdge {
    pub from: CubeId,
    pub to: CubeId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreOrder {
    pub dag: daggy::Dag<CubeId, PreOrderEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderStyle {
    /// Paragraph, e.g. text, text, text, ...
    Paragraph,
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
pub struct Order {
    pub style: OrderStyle,
    pub items: Vec<CubeId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Point {
    Text(Text),
    Image(Image),
    RawRef(RawRef),
    CubeRef(CubeRef),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextStyle {
    Paragraph,
    Heading { level: u8 },
    Todo { done: bool },
    Quote,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Text {
    pub style: TextStyle,
    pub text: RichText,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Image {
    pub blob_id: BlobId,
    pub caption: Option<RichText>,
    /// Provenance of the image.
    pub original_raw: Option<RawId>,
}

/// Shown as an embedded "card" of a raw entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawRef {
    pub raw_id: RawId,
}

/// An embedded "card" or "container" of a cube.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CubeRef {
    pub cube_id: CubeId,
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
