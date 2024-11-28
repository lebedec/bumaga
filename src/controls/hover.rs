use crate::{Control, ControlEvent, ControlTarget};
use serde_json::{json, Value};

pub struct HoverControl<Message> {
    id: usize,
    enter: Box<dyn HoverOutput<Message>>,
    leave: Box<dyn HoverOutput<Message>>,
}

pub trait HoverOutput<Message> {
    fn create(&self) -> Message;
}

pub fn c_hover<Message>(
    enter: impl HoverOutput<Message> + 'static,
    leave: impl HoverOutput<Message> + 'static,
) -> HoverControl<Message> {
    HoverControl::new(enter, leave)
}

impl<Message> HoverControl<Message> {
    pub fn new(
        enter: impl HoverOutput<Message> + 'static,
        leave: impl HoverOutput<Message> + 'static,
    ) -> Self {
        Self {
            id: 0,
            enter: Box::new(enter),
            leave: Box::new(leave),
        }
    }

    pub fn bind(&self) -> Value {
        json!({
            "id": self.id
        })
    }

    pub fn handle(&mut self, event: ControlEvent, _target: ControlTarget) -> Option<Message> {
        match event {
            ControlEvent::OnMouseEnter => {
                return Some(self.enter.create());
            }
            ControlEvent::OnMouseLeave => {
                return Some(self.leave.create());
            }
            _ => {}
        }
        None
    }
}

impl<Message> Control<Message> for HoverControl<Message> {
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
