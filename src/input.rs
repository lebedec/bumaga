use std::collections::HashMap;
use std::time::Duration;

use log::error;
use serde_json::{Map, Value};

use crate::{FontFace, Transformer};

pub struct Input<'f> {
    pub(crate) fonts: Option<&'f mut dyn Fonts>,
    pub(crate) value: Map<String, Value>,
    pub(crate) time: Duration,
    pub(crate) viewport: [f32; 2],
    pub(crate) events: Vec<InputEvent>,
}

impl<'f> Input<'f> {
    pub fn new() -> Input<'f> {
        Input {
            fonts: None,
            value: Map::new(),
            time: Duration::from_micros(0),
            viewport: [800.0, 600.0],
            events: vec![],
        }
    }

    pub fn fonts(mut self, fonts: &'f mut dyn Fonts) -> Self {
        self.fonts = Some(fonts);
        self
    }

    pub fn value(mut self, value: Value) -> Self {
        self.value = match value {
            Value::Object(value) => value,
            _ => {
                error!("unable to set input value, must be JSON object");
                return self;
            }
        };
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

    pub fn events(mut self, events: Vec<InputEvent>) -> Self {
        self.events = events;
        self
    }

    pub fn event(mut self, event: InputEvent) -> Self {
        self.events.push(event);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputEvent {
    Unknown,
    MouseMove([f32; 2]),
    MouseButtonDown(MouseButtons),
    MouseButtonUp(MouseButtons),
    MouseWheel([f32; 2]),
    KeyDown(Keys),
    KeyUp(Keys),
    Char(char),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseButtons {
    Left,
    Right,
}

/// It's hard to full match scancode or keycode from different platforms or windowing frameworks.
/// Bumaga encodes only most usable "control" keys which are responsible for application logic or text editing.
/// Any other "printable" keys must be passed as unicode characters in context of the current keyboard layout.
///
/// see for details: https://developer.mozilla.org/en-US/docs/Web/API/UI_Events/Keyboard_event_key_values
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Keys {
    Unknown,
    // UI keys
    Escape,
    // Editing keys
    Backspace,
    Delete,
    Insert,
    // Whitespace keys
    Enter,
    Tab,
    // Navigation keys
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    End,
    Home,
    PageDown,
    PageUp,
    // Modifier keys
    Alt,
    CapsLock,
    Ctrl,
    Shift,
}

pub trait Fonts {
    fn measure(&mut self, text: &str, style: &FontFace, max_width: Option<f32>) -> [f32; 2];
}

pub(crate) struct DummyFonts;

impl Fonts for DummyFonts {
    fn measure(&mut self, text: &str, style: &FontFace, max_width: Option<f32>) -> [f32; 2] {
        // NOTE: incorrect implementation, approximately calculates the text size
        // you should provide your own Fonts implementation
        let width = text.len() as f32 * style.size * 0.75;
        match max_width {
            None => [width, style.size],
            Some(max_width) => {
                if max_width == 0.0 {
                    [0.0, 0.0]
                } else {
                    let lines = 1.0 + (width / max_width).floor();
                    [max_width, lines * style.size]
                }
            }
        }
    }
}
