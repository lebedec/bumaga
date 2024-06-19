use std::time::Duration;
use scraper::{Html};
use serde_json::Value;
use taffy::{Layout};
use crate::models::{Presentation, Rectangle};
use crate::rendering::{State};

pub struct Component {
    pub(crate) presentation: Presentation,
    pub(crate) html: Html,
    pub(crate) state: State,
}

pub struct Input {
    pub(crate) value: Value,
    pub(crate) time: Duration,
    pub(crate) keys: Vec<String>,
    pub(crate) mouse_position: [f32; 2],
    pub(crate) mouse_button_down: bool
}

/// It is a mechanism that allows a Bumaga component to request
/// interaction event handling in application.
pub struct Call {
    /// The identifier of event handler (function name probably).
    pub function: String,
    /// The JSON-like argument from component template.
    pub arguments: Vec<Value>
}

pub struct Element {
    pub layout: Layout,
    pub rectangle: Rectangle
}

pub struct Frame {
    pub calls: Vec<Call>,
    pub elements: Vec<Element>
}

