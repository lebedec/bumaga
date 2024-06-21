use std::time::Duration;

pub use lightningcss::properties::background::{
    BackgroundAttachment, BackgroundClip, BackgroundOrigin, BackgroundPosition, BackgroundRepeat,
    BackgroundSize,
};
pub use lightningcss::properties::border::LineStyle;
pub use lightningcss::properties::font::{FontStretchKeyword, FontStyle};
pub use lightningcss::properties::text::OverflowWrap;
pub use lightningcss::values::color::{CssColor, RGBA};
use scraper::{Html, Selector};
use serde_json::Value;
pub use taffy::Layout;

use crate::models::{Presentation, ViewId};
use crate::rendering::State;

/// Components are reusable parts of UI that define views,
/// handle user input and store UI state between interactions.
pub struct Component {
    pub(crate) presentation: Presentation,
    pub(crate) html: Html,
    pub(crate) state: State,
    pub(crate) body_selector: Selector,
}

pub struct Input<'f> {
    pub(crate) fonts: Option<&'f mut dyn Fonts>,
    pub(crate) value: Value,
    pub(crate) time: Duration,
    pub(crate) keys: Vec<String>,
    pub(crate) viewport: [f32; 2],
    pub(crate) mouse_position: [f32; 2],
    pub(crate) mouse_button_down: bool,
}

pub struct Output {
    pub calls: Vec<Call>,
    pub elements: Vec<Element>,
}

/// It is a mechanism that allows a Bumaga component to request
/// interaction event handling in application.
pub struct Call {
    /// The identifier of event handler (function name probably).
    pub function: String,
    /// The JSON-like arguments.
    pub arguments: Vec<Value>,
}

/// The most fundamental object for building a UI, Element contains layout and appearance.
/// Element maps directly to the native rectangle view equivalent on whatever graphics engine
/// your application is running on, whether is a SDL_RenderDrawRect, glBegin(GL_QUADS), etc.
#[derive(Clone)]
pub struct Element {
    /// The final result of a layout algorithm, describes size and position of element.
    pub layout: Layout,
    pub id: ViewId,
    pub html_element: Option<scraper::node::Element>,
    pub background: MyBackground,
    pub borders: Borders,
    /// The foreground color of element (most often text color).
    pub color: RGBA,
    /// The text inside an element.
    pub text: Option<String>,
    /// The different properties of an element's text font.
    pub text_style: TextStyle,
}

#[derive(Clone)]
pub struct Borders {
    pub top: Option<MyBorder>,
    pub bottom: Option<MyBorder>,
    pub right: Option<MyBorder>,
    pub left: Option<MyBorder>,
}

#[derive(Clone)]
pub struct MyBorder {
    pub width: f32,
    pub style: LineStyle,
    pub color: CssColor,
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

#[derive(Clone)]
pub struct TextStyle {
    /// The font family.
    pub font_family: String,
    /// The font size.
    pub font_size: f32,
    /// The font style.
    pub font_style: FontStyle,
    /// The font weight.
    pub font_weight: u16,
    /// The font stretch.
    pub font_stretch: FontStretchKeyword,
    /// The line height.
    pub line_height: f32,
    /// The text overflow wrap.
    pub wrap: OverflowWrap,
}

pub trait Fonts {
    fn measure(&mut self, text: &str, style: &TextStyle, max_width: Option<f32>) -> [f32; 2];
}
