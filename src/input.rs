use std::time::Duration;
use serde_json::Value;
use crate::api::Input;

impl Input {
    pub fn new() -> Input {
        Input {
            value: Value::Null,
            time: Duration::from_micros(0),
            keys: vec![],
            mouse_position: [0.0, 0.0],
            mouse_button_down: false,
        }
    }

    pub fn value(mut self, value: Value) -> Input {
        self.value = value;
        self
    }

    pub fn time(mut self, time: Duration) -> Input {
        self.time = time;
        self
    }

    pub fn mouse(mut self, mouse_position: [f32; 2], down: bool) -> Input {
        self.mouse_position = mouse_position;
        self.mouse_button_down = down;
        self
    }
}