use crate::{Element, Input, InputEvent, MouseButtons, ViewError};
use log::error;
use pest::pratt_parser::Op;
use serde::de::Unexpected::Str;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, HashMap};
use taffy::{NodeId, TaffyTree};

pub type Bindings = BTreeMap<String, Vec<Binding>>;

pub trait Transformer: Fn(Value) -> Value {}

pub struct ViewModel {
    bindings: Bindings,
    model: Value,
    transformers: HashMap<String, Box<dyn Fn(Value) -> Value>>,
    // state
    pub(crate) focus: Option<NodeId>,
    mouse: [f32; 2],
}

impl ViewModel {
    pub fn create(bindings: Bindings, model: Value) -> Self {
        Self {
            bindings,
            model,
            transformers: HashMap::new(),
            focus: None,
            mouse: [0.0, 0.0],
        }
    }

    pub fn bind(
        &mut self,
        value: &Value,
        transformers: &HashMap<String, &dyn Fn(Value) -> Value>,
    ) -> Vec<Reaction> {
        let mut reactions = vec![];
        Self::bind_value(
            &mut self.model,
            value,
            "",
            &self.bindings,
            &mut reactions,
            transformers,
        );
        reactions
    }

    pub fn bind_value(
        mut dst: &mut Value,
        src: &Value,
        path: &str,
        bindings: &Bindings,
        reactions: &mut Vec<Reaction>,
        transformers: &HashMap<String, &dyn Fn(Value) -> Value>,
    ) {
        match (&mut dst, src) {
            (Value::Array(current), Value::Array(next)) => {
                if current.len() != next.len() {
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
            (Value::Object(_), _) => {
                error!("unable to bind '{path}', must be object")
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
        transformers: &HashMap<String, &dyn Fn(Value) -> Value>,
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
    ) -> Result<Vec<Call>, ViewError> {
        for event in input.events.iter() {
            match *event {
                InputEvent::MouseMove(mouse) => {
                    self.mouse = mouse;
                }
                _ => {}
            }
        }
        let mut output = vec![];
        self.capture_element_events(body, &input.events, &mut output, tree);
        Ok(output)
    }

    fn capture_element_events(
        &mut self,
        node: NodeId,
        events: &Vec<InputEvent>,
        output: &mut Vec<Call>,
        tree: &mut TaffyTree<Element>,
    ) {
        let element = tree.get_node_context_mut(node).unwrap();
        for event in events {
            match *event {
                InputEvent::MouseMove(cursor) => {
                    let hover = hovers(cursor, element);
                    if hover {
                        if !element.state.hover {
                            // fire enter
                            //println!("enter {}", element.tag);
                        }
                        element.state.hover = true;
                    } else {
                        if element.state.hover {
                            // fire leave
                            //println!("leave {}", element.tag);
                        }
                        element.state.hover = false;
                    }
                }
                InputEvent::MouseButtonDown(button) => {
                    if button == MouseButtons::Left && element.state.hover {
                        element.state.active = true;
                    }
                }
                InputEvent::MouseButtonUp(button) => {
                    if button == MouseButtons::Left {
                        element.state.active = false;
                        if element.state.hover {
                            if let Some(handler) = element.listeners.get("onclick") {
                                let value = if handler.argument == Schema::THIS {
                                    json!("<this>")
                                } else {
                                    self.model
                                        .pointer(&handler.argument)
                                        .cloned()
                                        .unwrap_or(Value::Null)
                                };
                                output.push(Call {
                                    function: handler.function.clone(),
                                    arguments: vec![value],
                                })
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        for child in tree.children(node).unwrap() {
            self.capture_element_events(child, events, output, tree);
        }
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
    Visibility(NodeId, bool),
    Attribute(NodeId, String),
    Repeat(NodeId, usize, usize),
}

impl Binding {
    fn react_value_change(&self, value: &Value) -> Reaction {
        match self.params.clone() {
            BindingParams::Visibility(node, visible) => {
                let visible = as_boolean(value) == visible;
                Reaction::Reattach { node, visible }
            }
            BindingParams::Attribute(node, key) => {
                let value = as_string(value);
                Reaction::Bind { node, key, value }
            }
            BindingParams::Text(node, span) => {
                let text = as_string(value);
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
        value: String,
    },
}

pub fn as_boolean(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(value) => *value,
        Value::Number(number) => number.as_f64().map(|value| value != 0.0).unwrap_or(false),
        Value::String(string) => string.len() > 0,
        Value::Array(array) => array.len() > 0,
        Value::Object(_) => true,
    }
}

pub fn as_string(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(value) => value.to_string(),
        Value::Number(number) => number.to_string(),
        Value::String(string) => string.clone(),
        Value::Array(_) => "[array]".to_string(),
        Value::Object(_) => "{object}".to_string(),
    }
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
