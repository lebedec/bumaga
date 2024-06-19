use std::time::Duration;
use serde_json::Value;

pub struct Component {

}

impl Component {
    pub fn new() -> Component {
        unimplemented!()
    }
    
    pub fn update(&mut self, input: Input) -> Frame {
        unimplemented!()
    }
}

pub struct Input {
    value: Value,
    time: Duration,
    keys: Vec<String>,
    mouse_position: [f32; 2]
}

/// It is a mechanism that allows a Bumaga component to request
/// interaction event handling in application.
pub struct Call {
    /// The identifier of event handler (function name probably).
    function: String,
    /// The JSON-like argument from component template.
    arguments: Vec<Value>
}

pub struct Element {

}

pub struct Frame {
    calls: Vec<Call>,
    elements: Vec<Element>
}

