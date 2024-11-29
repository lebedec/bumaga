use serde_json::Value;

#[derive(Debug, Default)]
pub struct Output {
    pub is_input_captured: bool,
    pub messages: Vec<Value>,
}

impl Output {
    pub fn new() -> Self {
        Self::default()
    }
}
