use std::time::Duration;

use serde_json::Value;

use crate::api::{Fonts, Input};
use crate::TextStyle;

impl<'f> Input<'f> {
    pub fn new() -> Input<'f> {
        Input {
            fonts: None,
            value: Value::Null,
            time: Duration::from_micros(0),
            keys: vec![],
            viewport: [800.0, 600.0],
            mouse_position: [0.0, 0.0],
            mouse_button_down: false,
        }
    }

    pub fn fonts(mut self, fonts: &'f mut dyn Fonts) -> Input<'f> {
        self.fonts = Some(fonts);
        self
    }

    pub fn value(mut self, value: Value) -> Input<'f> {
        self.value = value;
        self
    }

    pub fn time(mut self, time: Duration) -> Input<'f> {
        self.time = time;
        self
    }

    pub fn viewport(mut self, viewport: [f32; 2]) -> Input<'f> {
        self.viewport = viewport;
        self
    }

    pub fn mouse(mut self, mouse_position: [f32; 2], down: bool) -> Input<'f> {
        self.mouse_position = mouse_position;
        self.mouse_button_down = down;
        self
    }
}

pub(crate) struct FakeFonts;

impl Fonts for FakeFonts {
    fn measure(&mut self, text: &str, style: &TextStyle, max_width: Option<f32>) -> [f32; 2] {
        // NOTE: incorrect implementation, approximately calculates the text size
        // you should provide your own Fonts implementation
        let width = text.len() as f32 * style.font_size * 0.75;
        match max_width {
            None => [width, style.font_size],
            Some(max_width) => {
                if max_width == 0.0 {
                    [0.0, 0.0]
                } else {
                    let lines = 1.0 + (width / max_width).floor();
                    [max_width, lines * style.font_size]
                }
            }
        }
    }
}
