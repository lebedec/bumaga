use crate::{Control, ControlEvent, ControlTarget, Controls, InputEvent};
use serde_json::{json, Value};

pub struct ButtonControl<Message> {
    id: usize,
    output: Box<dyn ButtonOutput<Message>>,
    enabled: bool,
}

pub trait ButtonOutput<Message> {
    fn create(&self) -> Message;
}

pub fn c_button<Message>(output: impl ButtonOutput<Message> + 'static) -> ButtonControl<Message> {
    ButtonControl::new(output)
}

impl<Message> ButtonControl<Message> {
    pub fn new(output: impl ButtonOutput<Message> + 'static) -> Self {
        let mut control = Self {
            id: 0,
            output: Box::new(output),
            enabled: true,
        };
        control
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn bind(&self) -> Value {
        json!({
            "id": self.id
        })
    }

    pub fn handle(&mut self, event: ControlEvent, _target: ControlTarget) -> Option<Message> {
        match event {
            ControlEvent::OnClick => {
                if self.enabled {
                    return Some(self.output.create());
                }
            }
            _ => {}
        }
        None
    }
}

impl<Message> Control<Message> for ButtonControl<Message> {
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
