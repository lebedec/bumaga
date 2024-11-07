use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct Input {
    pub(crate) time: Duration,
    pub(crate) viewport: [f32; 2],
    pub(crate) events: Vec<InputEvent>,
}

impl<'f> Input {
    pub fn new() -> Input {
        Input {
            time: Duration::from_micros(0),
            viewport: [800.0, 600.0],
            events: vec![],
        }
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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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
