use crate::controls::SliderControl;
use crate::{Element, ElementState, InputEvent, Keys};
use log::error;
use serde_json::Value;
use std::collections::HashMap;
use std::mem::take;
use std::ops::{Add, AddAssign};

pub trait Control<Message> {
    fn identify(&mut self, id: usize);
    fn handle(&mut self, event: ControlEvent, target: ControlTarget) -> Option<Message>;
    fn bind(&self) -> Value;
}

pub struct Controls<Message> {
    pub controls: HashMap<usize, Box<dyn Control<Message>>>,
}

impl<Message> Controls<Message> {
    pub fn new() -> Self {
        Controls {
            controls: Default::default(),
        }
    }

    pub fn add_many(&mut self, controls: Vec<impl Control<Message> + 'static>) -> Vec<Value> {
        let mut values = vec![];
        for contron in controls {
            let value = self.add(contron);
            values.push(value);
        }
        values
    }

    pub fn add(&mut self, mut control: impl Control<Message> + 'static) -> Value {
        let id = self.controls.len();
        control.identify(id);
        let binding = control.bind();
        self.controls.insert(id, Box::new(control));
        binding
    }

    pub fn handle(&mut self, responses: Vec<ViewResponse>) -> Vec<Message> {
        let mut messages = vec![];
        for response in responses {
            if let Some(control) = self.controls.get_mut(&response.id) {
                if let Some(message) = control.handle(response.event, response.target) {
                    messages.push(message);
                }
            } else {
                error!("unable to handle view event, control not found")
            }
        }
        self.controls.clear();
        messages
    }
}

#[derive(Debug)]
pub struct ViewResponse {
    pub id: usize,
    pub event: ControlEvent,
    pub target: ControlTarget,
}

#[derive(Debug)]
pub struct ControlTarget {
    pub mouse: [f32; 2],
    pub size: [f32; 2],
    pub position: [f32; 2],
    pub state: ElementState,
}

impl ControlTarget {
    pub fn create(element: &Element, mouse: [f32; 2]) -> Self {
        Self {
            mouse,
            size: element.size,
            position: element.position,
            state: element.state,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ControlEvent {
    OnDrop,
    OnDragStart,
    OnDragEnd,
    OnDragEnter,
    OnDragLeave,
    OnDragOver,
    OnClick,
    OnContextMenu,
    OnFocus,
    OnBlur,
    OnInput(char),
    OnKeyDown(Keys),
    OnKeyUp(Keys),
    OnMouseDown,
    OnMouseEnter,
    OnMouseLeave,
    OnMouseMove,
    OnMouseUp,
}
