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
