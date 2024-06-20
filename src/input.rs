use std::time::Duration;
use serde_json::Value;
use crate::api::{Input, Fonts};
use crate::models::TextStyle;

static FAKE_FONTS: DummyFonts = DummyFonts {};

impl<'f> Input<'f> {
    
    pub fn new(fonts: &mut dyn Fonts) -> Input {
        Input {
            fonts,
            value: Value::Null,
            time: Duration::from_micros(0),
            keys: vec![],
            mouse_position: [0.0, 0.0],
            mouse_button_down: false,
        }
    }
    
    pub fn fonts(mut self, fonts: &'f mut dyn Fonts) -> Input<'f> {
        self.fonts = fonts;
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

    pub fn mouse(mut self, mouse_position: [f32; 2], down: bool) -> Input<'f> {
        self.mouse_position = mouse_position;
        self.mouse_button_down = down;
        self
    }
}

struct DummyFonts;

impl Fonts for DummyFonts {
    fn measure(&mut self, text: &str, style: &TextStyle, max_width: Option<f32>) -> [f32; 2] {
        [text.len() as f32 * style.font_size, style.font_size]
    }
}