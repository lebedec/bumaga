use lightningcss::properties::background::{BackgroundAttachment, BackgroundClip, BackgroundOrigin, BackgroundPosition, BackgroundRepeat, BackgroundSize};
use lightningcss::rules::style::StyleRule;
use lightningcss::values::color::{CssColor, RGBA};
use scraper::node::Element;
use scraper::Selector;

#[derive(Clone, Copy)]
pub struct SizeContext {
    pub level: usize,
    pub root_font_size: f32,
    pub parent_font_size: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
}

#[derive(Clone)]
pub struct MyBackground {
    /// The background image.
    pub image: Option<String>,
    /// The background color.
    pub color: CssColor,
    /// The background position.
    pub position: BackgroundPosition,
    /// How the background image should repeat.
    pub repeat: BackgroundRepeat,
    /// The size of the background image.
    pub size: BackgroundSize,
    /// The background attachment.
    pub attachment: BackgroundAttachment,
    /// The background origin.
    pub origin: BackgroundOrigin,
    /// How the background should be clipped.
    pub clip: BackgroundClip,
}


#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ElementId {
    pub element_n: usize,
    pub hash: u64
}

#[derive(Clone)]
pub struct Rectangle {
    pub id: ElementId,
    pub element: Option<Element>,
    pub key: String,
    pub background: MyBackground,
    pub color: RGBA,
    pub font_size: f32,
    pub text: Option<String>,
}

pub struct Ruleset<'i> {
    pub selector: Selector,
    pub style: StyleRule<'i>,
}

pub struct Presentation {
    pub rules: Vec<Ruleset<'static>>,
}

