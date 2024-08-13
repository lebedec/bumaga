use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use serde_json::{Map, Value};
pub use taffy::Layout;
use taffy::{NodeId, TaffyTree};

pub use value::ValueExtensions;

use crate::css::Css;
pub use crate::error::ViewError;
use crate::html::{Binder, Html};
use crate::state::State;
use crate::{value, Element, TextStyle};

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
