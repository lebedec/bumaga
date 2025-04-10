use serde_json::Value;

use crate::Pointer;

#[derive(Debug, Default)]
pub struct Output {
    pub is_input_captured: bool,
    pub pointer: Pointer,
    pub messages: Vec<Value>,
}

impl Output {
    pub fn new() -> Self {
        Self::default()
    }
}
