use crate::controls::{Control, Controls};
use crate::{ControlEvent, ControlTarget, InputEvent};
use serde_json::{json, Value};
use std::hash::Hash;

pub struct SliderControl<Message> {
    id: usize,
    horizontal: bool,
    output: Box<dyn SliderOutput<Message>>,
    initial_value: f32,
    value: f32,
    min: f32,
    max: f32,
}

pub trait SliderOutput<Message> {
    fn create(&self, value: f32) -> Message;
}

pub fn c_slider<Message>(
    value: impl ToFloat,
    min: impl ToFloat,
    max: impl ToFloat,
    output: impl SliderOutput<Message> + 'static,
) -> SliderControl<Message> {
    SliderControl::new(value.to_float(), min.to_float(), max.to_float(), output)
}

impl<Message> SliderControl<Message> {
    pub fn new(
        value: f32,
        min: f32,
        max: f32,
        output: impl SliderOutput<Message> + 'static,
    ) -> Self {
        let mut control = Self {
            id: 0,
            horizontal: true,
            output: Box::new(output),
            initial_value: value,
            value,
            min,
            max,
        };
        control.fix_value();
        control
    }

    pub fn vertical(mut self) -> Self {
        self.horizontal = false;
        self
    }

    pub fn bind(&self) -> Value {
        json!({
            "id": self.id,
            "value": self.value,
            "thumb": (self.value / (self.max - self.min)) * 100.0,
            "min": self.min,
            "max": self.max
        })
    }

    pub fn handle(&mut self, event: ControlEvent, target: ControlTarget) -> Option<Message> {
        match event {
            ControlEvent::OnMouseDown => {
                self.set_value_from_view(target);
            }
            ControlEvent::OnMouseMove => {
                if target.state.active {
                    self.set_value_from_view(target);
                }
            }
            _ => {}
        }
        if self.initial_value != self.value {
            Some(self.output.create(self.value))
        } else {
            None
        }
    }

    pub fn fix_value(&mut self) {
        let value = self.initial_value;
        let value = value.max(self.min);
        let value = value.min(self.max);
        self.value = value;
    }

    pub fn set_value_from_view(&mut self, target: ControlTarget) {
        let mouse_position_normalized = [
            (target.mouse[0] - target.position[0]) / target.size[0],
            (target.mouse[1] - target.position[1]) / target.size[1],
        ];
        let thumb = if self.horizontal {
            mouse_position_normalized[0]
        } else {
            mouse_position_normalized[1]
        };
        self.value = self.min + (self.max - self.min) * thumb;
    }
}

impl<Message> Control<Message> for SliderControl<Message> {
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

trait ToFloat {
    fn to_float(&self) -> f32;
}

impl ToFloat for i32 {
    fn to_float(&self) -> f32 {
        *self as f32
    }
}

impl ToFloat for f32 {
    fn to_float(&self) -> f32 {
        *self
    }
}

impl ToFloat for u8 {
    fn to_float(&self) -> f32 {
        *self as f32
    }
}
