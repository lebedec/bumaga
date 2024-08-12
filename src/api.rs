use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use serde_json::{Map, Value};
pub use taffy::Layout;
use taffy::{NodeId, TaffyTree};

pub use value::ValueExtensions;

use crate::animation::{Animator, Transition};
use crate::css::{Css, PropertyKey};
pub use crate::error::ComponentError;
use crate::html::{Binder, Html};
use crate::models::{ElementId, Object};
use crate::state::State;
use crate::styles::Scrolling;
use crate::value;

/// Components are reusable parts of UI that define views,
/// handle user input and store UI state between interactions.
pub struct Component {
    pub(crate) css: Source<Css>,
    pub(crate) html: Source<Html>,
    pub(crate) state: State,
    pub(crate) resources: String,
    pub(crate) tree: TaffyTree<Element>,
    pub(crate) root: NodeId,
}

pub struct Source<T> {
    pub(crate) path: Option<PathBuf>,
    pub(crate) modified: SystemTime,
    pub(crate) content: T,
}

pub struct Input<'f> {
    pub(crate) fonts: Option<&'f mut dyn Fonts>,
    pub(crate) value: Map<String, Value>,
    pub(crate) time: Duration,
    pub(crate) keys: Vec<String>,
    pub(crate) viewport: [f32; 2],
    pub(crate) mouse_position: [f32; 2],
    pub(crate) mouse_buttons_down: Vec<MouseButton>,
    pub(crate) mouse_buttons_up: Vec<MouseButton>,
    pub(crate) mouse_wheel: [f32; 2],
    pub(crate) keys_down: Vec<Keys>,
    pub(crate) keys_up: Vec<Keys>,
    pub(crate) keys_pressed: Vec<Keys>,
    pub(crate) characters: Vec<char>,
    pub(crate) transformers: HashMap<String, Box<dyn Fn(Value) -> Value>>,
}

pub struct Output {
    pub hover: Option<NodeId>,
    pub scroll: Option<NodeId>,
    pub calls: Vec<CallOld>,
    pub elements: Vec<Element>,
}

/// It is a mechanism that allows a Bumaga component to request
/// interaction event handling in application.
#[derive(Debug, Clone)]
pub struct CallOld {
    /// The identifier of event handler (function name probably).
    pub function: String,
    /// The JSON-like arguments.
    pub arguments: Vec<Value>,
}

#[derive(Debug, Clone)]
pub struct Handler {
    pub function: String,
    pub argument: Binder,
}

impl CallOld {
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
    pub html: Object,
    pub children: Vec<NodeId>,
    //
    pub tag: String,
    pub text: Option<TextContent>,
    pub attrs: HashMap<String, String>,
    pub pseudo_classes: HashSet<String>,
    //
    pub object_fit: ObjectFit,
    pub background: Background,
    pub borders: Borders,
    /// The foreground color of element (most often text color).
    pub color: Rgba,
    /// The different properties of an element's text font.
    pub text_style: TextStyle,
    pub listeners_old: HashMap<String, CallOld>,
    pub listeners: HashMap<String, Handler>,
    pub opacity: f32,
    pub transforms: Vec<TransformFunction>,
    pub animator: Animator,
    pub scrolling: Option<Scrolling>,
    pub clip: Option<Layout>,
    pub transitions: HashMap<PropertyKey, Transition>,
}

#[derive(Clone)]
pub struct TextContent {
    pub spans: Vec<String>,
}

#[derive(Clone, Copy, Debug)]
pub enum Length {
    Number(f32),
    Percent(f32),
}

impl Length {
    #[inline(always)]
    pub fn resolve(&self, base: f32) -> f32 {
        match *self {
            Length::Number(value) => value,
            Length::Percent(value) => value * base,
        }
    }

    pub fn px(value: f32) -> Self {
        Self::Number(value)
    }

    pub fn percent(value: f32) -> Self {
        Self::Percent(value)
    }

    pub fn zero() -> Self {
        Self::Number(0.0)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TransformFunction {
    Translate { x: Length, y: Length, z: f32 },
}

impl TransformFunction {
    pub fn translate(x: Length, y: Length, z: f32) -> Self {
        Self::Translate { x, y, z }
    }
}

pub type Rgba = [u8; 4];

#[derive(Clone)]
pub struct Borders {
    pub top: MyBorder,
    pub bottom: MyBorder,
    pub right: MyBorder,
    pub left: MyBorder,
    pub radius: [Length; 4],
}

impl Borders {
    pub fn top(&self) -> Option<MyBorder> {
        if self.top.width > 0.0 {
            Some(self.top)
        } else {
            None
        }
    }

    pub fn right(&self) -> Option<MyBorder> {
        if self.right.width > 0.0 {
            Some(self.right)
        } else {
            None
        }
    }

    pub fn bottom(&self) -> Option<MyBorder> {
        if self.bottom.width > 0.0 {
            Some(self.bottom)
        } else {
            None
        }
    }

    pub fn left(&self) -> Option<MyBorder> {
        if self.left.width > 0.0 {
            Some(self.left)
        } else {
            None
        }
    }
}

#[derive(Clone, Default, Copy)]
pub struct MyBorder {
    pub width: f32,
    pub color: Rgba,
}

#[derive(Clone)]
pub struct Background {
    /// The background image.
    pub image: Option<String>,
    /// The background color.
    pub color: Rgba,
    // The background position.
    // pub position: BackgroundPosition,
    // /// How the background image should repeat.
    // pub repeat: BackgroundRepeat,
    // /// The size of the background image.
    // pub size: BackgroundSize,
    // /// The background attachment.
    // pub attachment: BackgroundAttachment,
    // /// The background origin.
    // pub origin: BackgroundOrigin,
    // /// How the background should be clipped.
    // pub clip: BackgroundClip,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ObjectFit {
    Contain,
    Cover,
    Fill,
    None,
    ScaleDown,
}

#[derive(Clone, Debug)]
pub struct TextStyle {
    /// The font family.
    pub font_family: String,
    /// The font size.
    pub font_size: f32,
    // The font style.
    // pub font_style: FontStyle,
    /// The font weight.
    pub font_weight: u16,
    // The font stretch.
    // pub font_stretch: FontStretchKeyword,
    /// The line height.
    pub line_height: f32,
    // The text overflow wrap.
    // pub wrap: OverflowWrap,
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
