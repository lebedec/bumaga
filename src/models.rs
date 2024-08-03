use std::collections::{HashMap, HashSet};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::rc::Rc;

use lightningcss::rules::style::StyleRule;

use crate::animation::Animation;
use crate::html::Dom;
use crate::Element;

#[derive(Clone, Copy)]
pub struct SizeContext {
    pub root_font_size: f32,
    pub parent_font_size: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ElementId {
    pub pos: (usize, usize),
    pub hash: u64,
}

impl ElementId {
    /// Fake elements like caret or input text can't participate in user interaction or share
    /// any state. We cane safely use one "zero" id for these elements.
    pub fn fake() -> Self {
        Self {
            pos: (0, 0),
            hash: 0,
        }
    }

    /// Document object position is line and column position in source HTML file.
    /// It guarantees unique identification of original element.
    pub fn from(dom: &Dom) -> Self {
        Self {
            pos: dom.pos,
            hash: 0,
        }
    }

    pub fn child(other: ElementId, value: u64) -> Self {
        Self {
            pos: other.pos,
            hash: value,
        }
    }

    pub fn hash(dom: &Dom, value: impl Hash) -> Self {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        Self {
            pos: dom.pos,
            hash: hasher.finish(),
        }
    }
}

#[derive(Clone)]
pub struct Object {
    pub tag: String,
    pub attrs: HashMap<String, String>,
    pub text: Option<String>,
    pub pseudo_classes: HashSet<String>,
}

impl Object {
    pub fn text(text: String) -> Self {
        Self {
            tag: "".to_string(),
            attrs: Default::default(),
            text: Some(text),
            pseudo_classes: Default::default(),
        }
    }

    pub fn fake() -> Self {
        Self {
            tag: "".to_string(),
            attrs: Default::default(),
            text: None,
            pseudo_classes: Default::default(),
        }
    }
}

pub struct Ruleset<'i> {
    pub style: StyleRule<'i>,
}

pub struct Presentation {
    pub rules: Vec<Ruleset<'static>>,
    pub animations: HashMap<String, Rc<Animation>>,
}
