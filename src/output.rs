use crate::ViewResponse;

#[derive(Debug, Default)]
pub struct Output {
    pub is_input_captured: bool,
    pub responses: Vec<ViewResponse>,
}

impl Output {
    pub fn new() -> Self {
        Self::default()
    }
}
