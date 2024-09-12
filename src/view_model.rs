use crate::{
    Element, Input, InputEvent, Keys, MouseButtons, Output, PointerEvents, ValueExtensions,
    ViewError,
};
use log::error;

use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::time::Duration;
use taffy::{NodeId, TaffyTree};

pub type Bindings = BTreeMap<String, Vec<Binding>>;

pub type Transformer = fn(Value) -> Value;

pub struct ViewModel {
    bindings: Bindings,
    model: Value,
    pub(crate) transformers: HashMap<String, Transformer>,
    // state
    // pub(crate) focus: Option<NodeId>,
    pub(crate) mouse: [f32; 2],
    pub(crate) mouse_hovers: HashSet<NodeId>,
}

impl ViewModel {
    pub fn create(bindings: Bindings, model: Value) -> Self {
        Self {
            bindings,
            model,
            transformers: default_transformers(),
            mouse: [0.0, 0.0],
            mouse_hovers: HashSet::new(),
        }
    }

    pub fn bind(&mut self, value: &Value) -> Vec<Reaction> {
        let mut reactions = vec![];
        Self::bind_value(
            &mut self.model,
            value,
            "",
            &self.bindings,
            &mut reactions,
            &self.transformers,
        );
        reactions
    }

    pub fn bind_value(
        mut dst: &mut Value,
        src: &Value,
        path: &str,
        bindings: &Bindings,
        reactions: &mut Vec<Reaction>,
        transformers: &HashMap<String, Transformer>,
    ) {
        match (&mut dst, src) {
            (Value::Array(current), Value::Array(next)) => {
                if current != next {
                    current.resize(next.len(), Value::Null);
                    Self::react(path, src, bindings, reactions, transformers);
                }
                for (index, dst) in current.iter_mut().enumerate() {
                    let src = &next[index];
                    let path = format!("{path}/{index}");
                    Self::bind_value(dst, src, &path, bindings, reactions, transformers);
                }
            }
            (Value::Array(_), _) => {
                error!("unable to bind '{path}', must be array")
            }
            (Value::Object(object), Value::Object(src)) => {
                for (key, dst) in object.iter_mut() {
                    let path = format!("{path}/{key}");
                    let src = match src.get(key) {
                        Some(src) => src,
                        None => {
                            error!("unable to bind '{path}', must be specified");
                            continue;
                        }
                    };
                    Self::bind_value(dst, src, &path, bindings, reactions, transformers);
                }
            }
            (Value::Object(object), Value::Null) => {
                for (key, dst) in object.iter_mut() {
                    let path = format!("{path}/{key}");
                    Self::bind_value(dst, &Value::Null, &path, bindings, reactions, transformers);
                }
            }
            (dst, src) => {
                if *dst != src {
                    **dst = src.clone();
                    Self::react(path, src, bindings, reactions, transformers);
                }
            }
        }
    }

    #[inline]
    fn react(
        path: &str,
        value: &Value,
        bindings: &Bindings,
        reactions: &mut Vec<Reaction>,
        transformers: &HashMap<String, Transformer>,
    ) {
        if let Some(bindings) = bindings.get(path) {
            for binding in bindings {
                if binding.pipe.len() > 0 {
                    let mut value = value.clone();
                    for name in &binding.pipe {
                        match transformers.get(name) {
                            None => {
                                error!("unable to bind value, transformer {name} not found")
                            }
                            Some(transform) => {
                                value = transform(value);
                            }
                        }
                    }
                    reactions.push(binding.react_value_change(&value));
                } else {
                    reactions.push(binding.react_value_change(value))
                }
            }
        }
    }

    pub fn handle_output(
        &mut self,
        input: &Input,
        body: NodeId,
        tree: &mut TaffyTree<Element>,
    ) -> Result<Output, ViewError> {
        for event in input.events.iter() {
            match *event {
                InputEvent::MouseMove(mouse) => {
                    self.mouse = mouse;
                }
                _ => {}
            }
        }
        let mut output = Output::new();
        self.capture_element_events(body, &input.events, &mut output, tree)?;
        output.is_cursor_over_view = !self.mouse_hovers.is_empty();

        Ok(output)
    }

    pub(crate) fn fire(&self, element: &Element, event: &str, this: Value, output: &mut Output) {
        if let Some(handler) = element.listeners.get(event) {
            let mut value = if handler.argument == Schema::THIS {
                this
            } else {
                self.model
                    .pointer(&handler.argument)
                    .cloned()
                    .unwrap_or(Value::Null)
            };
            for name in &handler.pipe {
                match self.transformers.get(name) {
                    Some(transform) => value = transform(value),
                    None => error!("unable to bind value, transformer {name} not found"),
                }
            }
            output.calls.push(Call {
                function: handler.function.clone(),
                arguments: vec![value],
            })
        }
    }

    fn capture_element_events(
        &mut self,
        node: NodeId,
        events: &Vec<InputEvent>,
        output: &mut Output,
        tree: &mut TaffyTree<Element>,
    ) -> Result<(), ViewError> {
        for event in events {
            let element = tree.get_node_context_mut(node).unwrap();
            match *event {
                InputEvent::Char(char) if element.state.focus => match element.tag.as_str() {
                    "input" => {
                        let value = element.state.as_input()?;
                        value.push(char);
                        let value = value.clone();
                        self.fire(element, "oninput", value.clone().into(), output);
                        self.update_input_value(element.node, value.clone(), tree)?;
                    }
                    _ => {}
                },
                InputEvent::KeyDown(_key) => {}
                InputEvent::KeyUp(key) if element.state.focus => {
                    match element.tag.as_str() {
                        "input" => {
                            if key == Keys::Enter {
                                let value = element.state.as_input()?;
                                let value = value.clone().into();
                                self.fire(element, "onchange", value, output);
                            }
                            if key == Keys::Backspace {
                                let value = element.state.as_input()?;
                                if value.pop().is_some() {
                                    let value = value.clone();
                                    self.fire(element, "oninput", value.clone().into(), output);
                                    self.update_input_value(element.node, value.clone(), tree)?;
                                }
                            }
                        }
                        _ => {}
                    }
                    if key == Keys::Tab {
                        // next focus
                    }
                }
                event if element.pointer_events == PointerEvents::Auto => match event {
                    InputEvent::MouseWheel(wheel) if element.state.hover => {
                        if let Some(scrolling) = element.scrolling.as_mut() {
                            scrolling.offset(wheel);
                        }
                    }
                    InputEvent::MouseMove(cursor) => {
                        let hover = hovers(cursor, element);
                        if hover {
                            if !element.state.hover {
                                self.fire(element, "onmouseenter", Value::Null, output);
                            }
                            // TODO: rework algorithm (user input not only way to change hover)
                            element.state.hover = true;
                            self.mouse_hovers.insert(element.node);
                        } else {
                            if element.state.hover {
                                self.fire(element, "onmouseleave", Value::Null, output);
                            }
                            element.state.hover = false;
                            self.mouse_hovers.remove(&element.node);
                        }
                    }
                    InputEvent::MouseButtonDown(button) => {
                        if button == MouseButtons::Left && element.state.hover {
                            element.state.active = true;
                        }
                        if element.state.hover {
                            if !element.state.focus {
                                // fire focus
                            }
                            element.state.focus = true;
                        } else {
                            if element.state.focus {
                                match element.tag.as_str() {
                                    "input" => {
                                        let this = match &element.state.as_input() {
                                            Ok(value) => Value::String(value.to_string()),
                                            _ => Value::Null,
                                        };
                                        self.fire(element, "onchange", this, output);
                                    }
                                    _ => {}
                                }
                                self.fire(element, "onblur", Value::Null, output);
                            }
                            element.state.focus = false;
                        }
                    }
                    InputEvent::MouseButtonUp(button) => {
                        if button == MouseButtons::Left {
                            element.state.active = false;
                            if element.state.hover {
                                self.fire(element, "onclick", Value::Null, output);
                                match element.tag.as_str() {
                                    "option" => {
                                        self.update_option_value(element.node, tree, output)?
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        for child in tree.children(node).unwrap() {
            self.capture_element_events(child, events, output, tree)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Binder {
    pub path: Vec<String>,
    pub pipe: Vec<String>,
}

impl Binder {
    pub fn to_string(&self) -> String {
        let path = self.path.join(".");
        if self.pipe.len() > 0 {
            let pipe = self.pipe.join(" | ");
            format!("{{ {path} | {pipe} }}")
        } else {
            format!("{{ {path} }}")
        }
    }

    /// JSON Pointer defines a string syntax for identifying a specific JSON value.
    /// For more information read [RFC6901](https://datatracker.ietf.org/doc/html/rfc6901)
    pub fn to_json_pointer(&self, locals: &HashMap<String, String>) -> String {
        let head = &self.path[0];
        let default = &format!("/{head}");
        let head = locals.get(head).unwrap_or(default);
        let tail = &self.path[1..];
        if tail.len() > 0 {
            format!("{head}/{}", tail.join("/"))
        } else {
            format!("{head}")
        }
    }
}

#[derive(Debug, Clone)]
pub struct Binding {
    pub params: BindingParams,
    pub pipe: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum BindingParams {
    Text(NodeId, usize),
    Visibility(NodeId, NodeId, bool),
    Attribute(NodeId, String),
    Repeat(NodeId, usize, usize),
}

impl Binding {
    fn react_value_change(&self, value: &Value) -> Reaction {
        match self.params.clone() {
            BindingParams::Visibility(parent, node, visible) => {
                let visible = value.as_boolean() == visible;
                Reaction::Reattach {
                    parent,
                    node,
                    visible,
                }
            }
            BindingParams::Attribute(node, key) => Reaction::Bind {
                node,
                key,
                value: value.clone(),
            },
            BindingParams::Text(node, span) => {
                let text = value.eval_string();
                Reaction::Type { node, span, text }
            }
            BindingParams::Repeat(parent, start, size) => {
                if let Some(value) = value.as_array() {
                    let count = value.len();
                    let count = if count > size {
                        error!("unable to repeat all items of {parent:?}");
                        size
                    } else {
                        count
                    };
                    Reaction::Repeat {
                        parent,
                        start,
                        cursor: start + count,
                        end: start + size,
                    }
                } else {
                    error!("unable to repeat, value must be array");
                    Reaction::Repeat {
                        parent,
                        start,
                        cursor: start,
                        end: start + size,
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum Reaction {
    Type {
        node: NodeId,
        span: usize,
        text: String,
    },
    Reattach {
        parent: NodeId,
        node: NodeId,
        visible: bool,
    },
    Repeat {
        parent: NodeId,
        start: usize,
        cursor: usize,
        end: usize,
    },
    Bind {
        node: NodeId,
        key: String,
        value: Value,
    },
}

fn hovers(point: [f32; 2], element: &Element) -> bool {
    let x = point[0] - element.position[0];
    let y = point[1] - element.position[1];
    x >= 0.0 && x <= element.size[0] && y >= 0.0 && y <= element.size[1]
}

pub struct Schema {
    pub value: Value,
}

impl Schema {
    const THIS: &'static str = "/this";

    pub fn new() -> Self {
        Self { value: json!({}) }
    }

    pub fn index(&mut self, binder: &Binder, i: usize, locals: &HashMap<String, String>) -> String {
        let pointer = binder.to_json_pointer(locals);
        let pointer = format!("{pointer}/{i}");
        Self::define_value(&mut self.value, &pointer);
        pointer
    }

    pub fn field(&mut self, binder: &Binder, locals: &HashMap<String, String>) -> String {
        let pointer = binder.to_json_pointer(locals);
        Self::define_value(&mut self.value, &pointer);
        pointer
    }

    fn define_value(mut target: &mut Value, pointer: &str) {
        if pointer == Schema::THIS {
            return;
        }
        for token in pointer.split('/').skip(1) {
            match token.parse::<usize>() {
                Ok(index) => {
                    if !target.is_array() {
                        *target = json!([]);
                    }
                    let array = target.as_array_mut().unwrap();
                    if array.len() <= index {
                        array.resize(index + 1, Value::Null);
                    }
                    target = &mut array[index];
                }
                _ => {
                    if !target.is_object() {
                        *target = json!({});
                    }
                    let object = target.as_object_mut().unwrap();
                    if !object.contains_key(token) {
                        object.insert(token.to_string(), Value::Null);
                    }
                    target = object.get_mut(token).unwrap();
                }
            }
        }
    }
}

/// It is a mechanism that allows a Bumaga component to request
/// interaction event handling in application.
#[derive(Debug, Clone)]
pub struct Call {
    /// The identifier of event handler (function name probably).
    pub function: String,
    /// The JSON-like arguments.
    pub arguments: Vec<Value>,
}

#[derive(Debug, Clone)]
pub struct Handler {
    pub function: String,
    pub argument: String,
    pub pipe: Vec<String>,
}

impl Call {
    pub fn signature(&self) -> (&str, &[Value]) {
        let name = self.function.as_str();
        let args = self.arguments.as_slice();
        (name, args)
    }

    pub fn get_str(&self, index: usize) -> Option<&str> {
        self.arguments.get(index).and_then(Value::as_str)
    }
}

fn default_transformers() -> HashMap<String, Transformer> {
    fn duration(value: Value) -> Value {
        match value.as_f64() {
            None => value,
            Some(value) => {
                let value = Duration::from_secs_f64(value);
                let value = format!("{value:?}");
                Value::String(value)
            }
        }
    }
    let mut transformers = HashMap::new();
    transformers.insert("duration".to_string(), duration as Transformer);
    transformers
}
