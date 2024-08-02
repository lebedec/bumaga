use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

pub use lightningcss::properties::background::{
    BackgroundAttachment, BackgroundClip, BackgroundOrigin, BackgroundPosition, BackgroundRepeat,
    BackgroundSize,
};
pub use lightningcss::properties::border::LineStyle;
pub use lightningcss::properties::font::{FontStretchKeyword, FontStyle};
pub use lightningcss::properties::text::OverflowWrap;
use lightningcss::properties::transform::Matrix3d;
use serde_json::Value;
pub use taffy::Layout;

use crate::html::Object;
use crate::models::{ElementId, Presentation};
use crate::state::State;
use crate::value;
pub use value::ValueExtensions;

/// Components are reusable parts of UI that define views,
/// handle user input and store UI state between interactions.
pub struct Component {
    pub(crate) presentation: Source<Presentation>,
    pub(crate) html: Source<Object>,
    pub(crate) state: State,
    pub(crate) resources: String,
}

pub struct Source<T> {
    pub(crate) path: Option<PathBuf>,
    pub(crate) modified: SystemTime,
    pub(crate) content: T,
}

pub struct Input<'f> {
    pub(crate) fonts: Option<&'f mut dyn Fonts>,
    pub(crate) value: Value,
    pub(crate) time: Duration,
    pub(crate) keys: Vec<String>,
    pub(crate) viewport: [f32; 2],
    pub(crate) mouse_position: [f32; 2],
    pub(crate) mouse_buttons_down: Vec<MouseButton>,
    pub(crate) mouse_buttons_up: Vec<MouseButton>,
    pub(crate) keys_down: Vec<Keys>,
    pub(crate) keys_up: Vec<Keys>,
    pub(crate) keys_pressed: Vec<Keys>,
    pub(crate) characters: Vec<char>,
    pub(crate) transformers: HashMap<String, Box<dyn Fn(Value) -> Value>>,
}

pub struct Output {
    pub calls: Vec<Call>,
    pub elements: Vec<Element>,
}

/// It is a mechanism that allows a Bumaga component to request
/// interaction event handling in application.
#[derive(Debug, Clone)]
pub struct Call {
    /// The identifier of event handler (function name probably).
    pub function: String,
    /// The JSON-like arguments.
    pub arguments: Vec<Value>,
}

impl Call {
    pub fn signature(&self) -> (&str, &[Value]) {
        let name = self.function.as_str();
        let args = self.arguments.as_slice();
        (name, args)
    }

    pub fn get_str(&self, index: usize) -> Option<&str> {
        self.arguments.get(index).and_then(Value::as_str)
    }
}

/// The most fundamental object for building a UI, Element contains layout and appearance.
/// Element maps directly to the native rectangle view equivalent on whatever graphics engine
/// your application is running on, whether is a SDL_RenderDrawRect, glBegin(GL_QUADS), etc.
#[derive(Clone)]
pub struct Element {
    /// The final result of a layout algorithm, describes size and position of element.
    pub layout: Layout,
    pub id: ElementId,
    pub html: Html,
    pub object_fit: ObjectFit,
    pub background: Background,
    pub borders: Borders,
    /// The foreground color of element (most often text color).
    pub color: Rgba,
    /// The different properties of an element's text font.
    pub text_style: TextStyle,
    pub listeners: HashMap<String, Call>,
    pub opacity: f32,
    pub transform: Option<Matrix3d<f32>>,
}

#[derive(Clone)]
pub struct Html {
    pub tag: String,
    pub attrs: HashMap<String, String>,
    pub text: Option<String>,
    pub pseudo_classes: HashSet<String>,
}

pub type Rgba = [u8; 4];

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
    pub color: Rgba,
}

#[derive(Clone)]
pub struct Background {
    /// The background image.
    pub image: Option<String>,
    /// The background color.
    pub color: Rgba,
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

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ObjectFit {
    Contain,
    Cover,
    Fill,
    None,
    ScaleDown,
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

/// It's hard to full match scancode or keycode from different platforms or windowing frameworks.
/// Bumaga encodes only most usable "control" keys which are responsible for application logic or text editing.
/// Any other "printable" keys must be passed as unicode characters in context of the current keyboard layout.
///
/// see for details: https://developer.mozilla.org/en-US/docs/Web/API/UI_Events/Keyboard_event_key_values
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Keys {
    Unknown,
    // UI keys
    Escape,
    // Editing keys
    Backspace,
    Delete,
    Insert,
    // Whitespace keys
    Enter,
    Tab,
    // Navigation keys
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    End,
    Home,
    PageDown,
    PageUp,
    // Modifier keys
    Alt,
    CapsLock,
    Ctrl,
    Shift,
}

pub type MouseButton = u16;

pub const LEFT_MOUSE_BUTTON: MouseButton = 0;

pub const RIGHT_MOUSE_BUTTON: MouseButton = 1;
