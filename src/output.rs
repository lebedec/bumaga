use crate::Call;

#[derive(Debug, Default)]
pub struct Output {
    pub calls: Vec<Call>,
    pub is_cursor_over_view: bool,
}

impl Output {
    pub fn new() -> Self {
        Self::default()
    }
}
