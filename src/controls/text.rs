use crate::controls::{Control, Controls};
use crate::{ControlEvent, ControlTarget, InputEvent, Keys};
use serde_json::{json, Value};

pub struct TextControl<Message> {
    id: usize,
    output: Box<dyn TextOutput<Message>>,
    initial_value: String,
    value: String,
}

pub trait TextOutput<Message> {
    fn create(&self, value: &str) -> Message;
}

pub fn c_text<Message>(
    value: impl ToString,
    output: impl TextOutput<Message> + 'static,
) -> TextControl<Message> {
    TextControl::new(value.to_string(), output)
}

impl<Message> TextControl<Message> {
    pub fn new(value: String, output: impl TextOutput<Message> + 'static) -> Self {
        let mut control = Self {
            id: 0,
            output: Box::new(output),
            initial_value: value.clone(),
            value,
        };
        control
    }

    pub fn bind(&self) -> Value {
        json!({
            "id": self.id,
            "value": self.value
        })
    }

    pub fn handle(&mut self, event: ControlEvent, _target: ControlTarget) -> Option<Message> {
        match event {
            ControlEvent::OnInput(char) => {
                self.value.push(char);
            }
            ControlEvent::OnKeyDown(key) => match key {
                Keys::Enter => {}
                Keys::Backspace => {
                    self.value.pop();
                }
                _ => {}
            },
            _ => {}
        }
        if self.initial_value != self.value {
            Some(self.output.create(&self.value))
        } else {
            None
        }
    }
}

impl<Message> Control<Message> for TextControl<Message> {
    fn identify(&mut self, id: usize) {
        self.id = id;
    }

    fn handle(&mut self, event: ControlEvent, target: ControlTarget) -> Option<Message> {
        self.handle(event, target)
    }

    fn bind(&self) -> Value {
        self.bind()
    }
}
