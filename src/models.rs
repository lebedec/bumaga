use std::collections::{HashMap, HashSet};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::rc::Rc;

use crate::html::Html;

#[derive(Clone, Copy)]
pub struct Sizes {
    pub root_font_size: f32,
    pub parent_font_size: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ElementId {
    pub(crate) index: usize,
    pub(crate) hash: u64,
}

impl ElementId {
    /// Fake elements like caret or input text can't participate in user interaction or share
    /// any state. We cane safely use one "zero" id for these elements.
    pub fn fake() -> Self {
        Self { index: 0, hash: 0 }
    }

    /// Document object position is line and column position in source HTML file.
    /// It guarantees unique identification of original element.
    pub fn from(dom: &Html) -> Self {
        Self {
            index: dom.index,
            hash: 0,
        }
    }

    pub fn child(other: ElementId, value: u64) -> Self {
        Self {
            index: other.index,
            hash: value,
        }
    }

    pub fn hash(dom: &Html, value: impl Hash) -> Self {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        Self {
            index: dom.index,
            hash: hasher.finish(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Object {
    pub tag: String,
    pub attrs: HashMap<String, String>,
    pub text: Option<String>,
    pub pseudo_classes: HashSet<String>,
}

impl Object {
    pub fn element(html: &Html) -> Self {
        Self {
            tag: html.tag.to_string(),
            attrs: html.attrs.clone(),
            text: None,
            pseudo_classes: Default::default(),
        }
    }

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

    pub fn tag(tag: &str) -> Self {
        Self {
            tag: tag.to_string(),
            attrs: Default::default(),
            text: None,
            pseudo_classes: Default::default(),
        }
    }
}
