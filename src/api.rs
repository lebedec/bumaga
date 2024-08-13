use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use serde_json::{Map, Value};
pub use taffy::Layout;
use taffy::{NodeId, TaffyTree};

pub use value::ValueExtensions;

use crate::css::Css;
pub use crate::error::ViewError;
use crate::html::Html;
use crate::state::State;
use crate::{value, Element, ElementFont};

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
