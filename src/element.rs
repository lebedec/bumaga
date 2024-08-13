use std::collections::HashMap;

use taffy::{Layout, NodeId};

use crate::animation::{Animator, Transition};
use crate::css::PropertyKey;
use crate::styles::Scrolling;
use crate::{Call, Handler};

/// The most fundamental object for building a UI, Element contains layout and appearance.
/// Element maps directly to the native rectangle view equivalent on whatever graphics engine
/// your application is running on, whether is a SDL_RenderDrawRect, glBegin(GL_QUADS), etc.
pub struct Element {
    pub node: NodeId,
    pub children: Vec<NodeId>,
    pub tag: String,
    pub text: Option<TextContent>,
    pub attrs: HashMap<String, String>,
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub object_fit: ObjectFit,
    pub background: Background,
    pub borders: Borders,
    /// The foreground color of element (most often text color).
    pub color: Rgba,
    /// The different properties of an element's text font.
    pub font: ElementFont,
    pub listeners: HashMap<String, Handler>,
    pub opacity: f32,
    pub transforms: Vec<TransformFunction>,
    pub animator: Animator,
    pub scrolling: Option<Scrolling>,
    pub clip: Option<Layout>,
    pub(crate) transitions: HashMap<PropertyKey, Transition>,
    pub(crate) state: ElementState,
}

#[derive(Clone)]
pub struct TextContent {
    pub spans: Vec<String>,
}

impl TextContent {
    #[inline(always)]
    pub fn as_string(&self) -> String {
        self.spans.join(" ")
    }
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
pub struct ElementFont {
    /// The font family.
    pub family: String,
    /// The font size.
    pub size: f32,
    // The font style.
    // pub font_style: FontStyle,
    /// The font weight.
    pub weight: u16,
    // The font stretch.
    // pub font_stretch: FontStretchKeyword,
    /// The line height.
    pub line_height: f32,
    // The text overflow wrap.
    // pub wrap: OverflowWrap,
}

#[derive(Default)]
pub struct ElementState {
    pub active: bool,
    pub hover: bool,
    pub value: Option<String>,
}
