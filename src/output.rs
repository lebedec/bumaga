use crate::Call;

pub struct Output {
    pub calls: Vec<Call>,
    pub is_cursor_over_view: bool,
}

impl Output {
    pub(crate) fn new() -> Self {
        Self {
            calls: vec![],
            is_cursor_over_view: false,
        }
    }
}
