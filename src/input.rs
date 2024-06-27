use std::collections::HashMap;
use std::time::Duration;

use serde_json::Value;

use crate::{Keys, MouseButton, TextStyle};
use crate::api::{Fonts, Input};

impl<'f> Input<'f> {
    pub fn new() -> Input<'f> {
        Input {
            fonts: None,
            value: Value::Null,
            time: Duration::from_micros(0),
            keys: vec![],
            viewport: [800.0, 600.0],
            mouse_position: [0.0, 0.0],
            mouse_buttons_down: vec![],
            mouse_buttons_up: vec![],
            keys_down: vec![],
            keys_up: vec![],
            keys_pressed: vec![],
            characters: vec![],
            transformers: HashMap::new(),
        }
    }

    pub fn pipe(mut self, name: &str, transformer: impl Fn(Value) -> Value + 'static) -> Self {
        self.transformers
            .insert(name.to_string(), Box::new(transformer));
        self
    }

    pub fn fonts(mut self, fonts: &'f mut dyn Fonts) -> Self {
        self.fonts = Some(fonts);
        self
    }

    pub fn value(mut self, value: Value) -> Self {
        self.value = value;
        self
    }

    pub fn time(mut self, time: Duration) -> Self {
        self.time = time;
        self
    }

    pub fn viewport(mut self, viewport: [f32; 2]) -> Self {
        self.viewport = viewport;
        self
    }

    pub fn mouse_buttons_down(mut self, mouse_buttons_down: Vec<MouseButton>) -> Self {
        self.mouse_buttons_down = mouse_buttons_down;
        self
    }

    pub fn mouse_buttons_up(mut self, mouse_buttons_up: Vec<MouseButton>) -> Self {
        self.mouse_buttons_up = mouse_buttons_up;
        self
    }

    pub fn mouse_position(mut self, mouse_position: [f32; 2]) -> Self {
        self.mouse_position = mouse_position;
        self
    }

    pub fn keys_down(mut self, keys_down: Vec<Keys>) -> Self {
        self.keys_down = keys_down;
        self
    }

    pub fn keys_up(mut self, keys_up: Vec<Keys>) -> Self {
        self.keys_up = keys_up;
        self
    }

    pub fn keys_pressed(mut self, keys_pressed: Vec<Keys>) -> Self {
        self.keys_pressed = keys_pressed;
        self
    }

    pub fn characters(mut self, characters: Vec<char>) -> Self {
        self.characters = characters;
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
