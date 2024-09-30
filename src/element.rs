use log::error;
use std::collections::{HashMap, HashSet};
use taffy::{Layout, NodeId};

use crate::animation::{Animator, Transition};
use crate::css::{Declaration, Property, PropertyKey};
use crate::styles::Scrolling;
use crate::{Handler, ViewError};

/// The most fundamental object for building a UI, Element contains layout and appearance.
/// Element maps directly to the native rectangle view equivalent on whatever graphics engine
/// your application is running on, whether is a SDL_RenderDrawRect, glBegin(GL_QUADS), etc.
pub struct Element {
    pub node: NodeId,
    pub children: Vec<NodeId>,
    pub tag: String,
    pub text: Option<TextContent>,
    pub attrs: HashMap<String, String>,
    pub attrs_bindings: HashMap<String, TextContent>,
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub content_size: [f32; 2],
    pub object_fit: ObjectFit,
    pub backgrounds: Vec<Background>,
    pub borders: Borders,
    /// The foreground color of element (most often text color).
    pub color: Rgba,
    /// The different properties of an element's text font.
    pub font: FontFace,
    pub listeners: HashMap<String, Handler>,
    pub opacity: f32,
    pub transforms: Vec<TransformFunction>,
    pub scrolling: Option<Scrolling>,
    pub clipping: Option<Layout>,
    pub pointer_events: PointerEvents,

    pub(crate) style: Vec<Declaration>,
    pub(crate) animators: Vec<Animator>,
    pub(crate) state: ElementState,
    pub(crate) transitions: Vec<Transition>,
}

impl Element {
    #[inline(always)]
    pub fn draggable(&self) -> bool {
        match self.attrs.get("draggable") {
            Some(value) => value == "true",
            None => false,
        }
    }

    #[inline(always)]
    pub fn value(&self) -> Option<&String> {
        self.attrs.get("value")
    }

    pub fn get_background_mut(&mut self, index: usize) -> &mut Background {
        if index >= self.backgrounds.len() {
            self.backgrounds.resize_with(index + 1, Background::default);
        }
        &mut self.backgrounds[index]
    }

    pub fn get_animator_mut(&mut self, index: usize) -> &mut Animator {
        if index >= self.animators.len() {
            self.animators.resize_with(index + 1, Animator::default);
        }
        &mut self.animators[index]
    }

    pub fn get_transition_mut(&mut self, index: usize) -> &mut Transition {
        if index >= self.transitions.len() {
            self.transitions.resize_with(index + 1, Transition::default);
        }
        &mut self.transitions[index]
    }
}

#[derive(Clone)]
pub struct TextContent {
    spans: Vec<String>,
}

impl TextContent {
    pub fn new(spans: Vec<String>) -> Self {
        Self { spans }
    }

    #[inline(always)]
    pub fn set(&mut self, span: usize, value: String) {
        if span >= self.spans.len() {
            error!(
                "unable to bind text span {span} to {value}, text consists of {} spans",
                self.spans.len()
            )
        } else {
            self.spans[span] = value;
        }
    }

    #[inline(always)]
    pub fn to_string(&self) -> String {
        self.spans.join("").trim().to_string()
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
    pub src: [f32; 2],
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

impl Default for Background {
    fn default() -> Self {
        Self {
            image: None,
            color: [0; 4],
            src: [0.0; 2],
        }
    }
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
pub struct FontFace {
    /// The font family.
    pub family: String,
    /// The font size.
    pub size: f32,
    // The font styles.
    pub style: String,
    /// The font weight.
    pub weight: u16,
    // The font stretch.
    // pub font_stretch: FontStretchKeyword,
    /// The line height.
    pub line_height: f32,
    // The text overflow wrap.
    // pub wrap: OverflowWrap,
    pub align: TextAlign,
}

#[derive(Clone, Debug)]
pub enum TextAlign {
    Start,
    End,
    Left,
    Right,
    Center,
    Justify,
    JustifyAll,
    MatchParent,
}

#[derive(Default, Clone, Copy)]
pub struct ElementState {
    pub active: bool,
    pub hover: bool,
    pub focus: bool,
    pub checked: bool,
}

#[derive(Default, PartialEq, Clone, Copy)]
pub enum PointerEvents {
    #[default]
    Auto,
    None,
}
