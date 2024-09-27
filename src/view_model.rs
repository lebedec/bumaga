use crate::{
    Element, Fragment, Input, InputEvent, Keys, MouseButtons, Output, PointerEvents,
    ValueExtensions, ViewError,
};
use log::error;

use crate::html::ArgumentBinding;
use pest::state;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::format;
use std::mem::take;
use std::time::Duration;
use taffy::{NodeId, TaffyTree};

pub type Bindings = BTreeMap<String, Vec<Binding>>;

pub type Transformer = fn(Value) -> Value;

pub struct ViewModel {
    pub(crate) bindings: Bindings,
    model: Value,
    model_array_default: HashMap<String, Value>,
    pub(crate) transformers: HashMap<String, Transformer>,
    // state
    // pub(crate) focus: Option<NodeId>,
    pub(crate) mouse: [f32; 2],
    pub(crate) mouse_hovers: HashSet<NodeId>,
    output: Output,
}

impl ViewModel {
    pub fn create(bindings: Bindings, model: Value) -> Self {
        let mut model_array_default = HashMap::new();
        Self::memorize_array_default("", &model, &mut model_array_default);
        Self {
            bindings,
            model,
            model_array_default,
            transformers: default_transformers(),
            mouse: [0.0, 0.0],
            mouse_hovers: HashSet::new(),
            output: Output::new(),
        }
    }

    fn memorize_array_default(
        type_path: &str,
        value: &Value,
        default: &mut HashMap<String, Value>,
    ) {
        match value {
            Value::Array(array) => {
                default.insert(type_path.to_string(), array[0].clone());
                Self::memorize_array_default(type_path, &array[0], default)
            }
            Value::Object(object) => {
                for (key, value) in object {
                    Self::memorize_array_default(&format!("{type_path}/{key}"), value, default)
                }
            }
            _ => {}
        }
    }

    pub fn bind(&mut self, value: &Value) -> Vec<Reaction> {
        let mut reactions = vec![];
        Self::bind_value(
            &mut self.model,
            value,
            "",
            "",
            &self.bindings,
            &mut reactions,
            &self.transformers,
            &self.model_array_default,
        );
        reactions
    }

    pub fn bind_value(
        mut dst: &mut Value,
        src: &Value,
        path: &str,
        arrays_path: &str,
        bindings: &Bindings,
        reactions: &mut Vec<Reaction>,
        transformers: &HashMap<String, Transformer>,
        default: &HashMap<String, Value>,
    ) {
        match (&mut dst, src) {
            (Value::Array(current), Value::Array(next)) => {
                if current.len() != next.len() {
                    if let Some(default) = default.get(arrays_path).cloned() {
                        current.resize(next.len(), default);
                        Self::react(path, src, bindings, reactions, transformers);
                    } else {
                        error!("unable to resize array {path} default not found");
                    }
                }
                for (index, dst) in current.iter_mut().enumerate() {
                    let src = &next[index];
                    let path = format!("{path}/{index}");
                    Self::bind_value(
                        dst,
                        src,
                        &path,
                        arrays_path,
                        bindings,
                        reactions,
                        transformers,
                        default,
                    );
                }
            }
            (Value::Array(_), _) => {
                error!("unable to bind '{path}', must be array")
            }
            (Value::Object(object), Value::Object(src)) => {
                for (key, dst) in object.iter_mut() {
                    let path = format!("{path}/{key}");
                    let arrays_path = format!("{arrays_path}/{key}");
                    let src = match src.get(key) {
                        Some(src) => src,
                        None => {
                            error!("unable to bind '{path}', must be specified");
                            continue;
                        }
                    };
                    Self::bind_value(
                        dst,
                        src,
                        &path,
                        &arrays_path,
                        bindings,
                        reactions,
                        transformers,
                        default,
                    );
                }
            }
            (Value::Object(object), Value::Null) => {
                for (key, dst) in object.iter_mut() {
                    let path = format!("{path}/{key}");
                    let arrays_path = format!("{arrays_path}/{key}");
                    Self::bind_value(
                        dst,
                        &Value::Null,
                        &path,
                        &arrays_path,
                        bindings,
                        reactions,
                        transformers,
                        default,
                    );
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
        self.output = Output::new();
        self.capture_hovers(body, tree)?;
        self.capture_element_events(body, tree, &input.events)?;
        self.output.is_cursor_over_view = !self.mouse_hovers.is_empty();
        Ok(take(&mut self.output))
    }

    fn capture_hovers(
        &mut self,
        body: NodeId,
        tree: &mut TaffyTree<Element>,
    ) -> Result<(), ViewError> {
        for node in self.mouse_hovers.clone() {
            let element = tree.get_node_context_mut(node).unwrap();
            let hover = hovers(self.mouse, element);
            if !hover {
                self.fire(element, "onmouseleave", Value::Null);
                element.state.hover = false;
                self.mouse_hovers.remove(&element.node);
            }
        }
        self.capture_element_hover(body, tree);
        Ok(())
    }

    fn capture_element_hover(&mut self, node: NodeId, tree: &mut TaffyTree<Element>) {
        let element = tree.get_node_context_mut(node).unwrap();
        let hover = hovers(self.mouse, element);
        if hover {
            if !element.state.hover {
                if element.pointer_events == PointerEvents::Auto {
                    self.fire(element, "onmouseenter", Value::Null);
                    element.state.hover = true;
                    self.mouse_hovers.insert(element.node);
                }
            }
            for child in tree.children(node).expect("childs must exist") {
                self.capture_element_hover(child, tree);
            }
        }
    }

    fn capture_element_events(
        &mut self,
        node: NodeId,
        tree: &mut TaffyTree<Element>,
        events: &Vec<InputEvent>,
    ) -> Result<(), ViewError> {
        for event in events {
            let element = tree
                .get_node_context_mut(node)
                .expect("node element must exist");
            match *event {
                InputEvent::Char(char) => match element.tag.as_str() {
                    "input" => self.handle_input_char(node, char, tree)?,
                    _ => {}
                },
                InputEvent::KeyDown(_key) => {}
                InputEvent::KeyUp(key) => match element.tag.as_str() {
                    "input" => self.handle_input_key_up(node, key, tree)?,
                    _ => {}
                },
                event if element.pointer_events == PointerEvents::Auto => match event {
                    InputEvent::MouseWheel(wheel) if element.state.hover => {
                        if let Some(scrolling) = element.scrolling.as_mut() {
                            scrolling.offset(wheel);
                        }
                    }
                    InputEvent::MouseButtonDown(button) => {
                        if button == MouseButtons::Left && element.state.hover {
                            element.state.active = true;
                        }
                        if element.state.hover {
                            if !element.state.focus {
                                self.fire(&element, "onfocus", Value::Null);
                            }
                            element.state.focus = true;
                        } else {
                            let focus_lost = take(&mut element.state.focus);
                            if focus_lost {
                                match element.tag.as_str() {
                                    "input" => self.handle_input_blur(node, tree)?,
                                    _ => {
                                        self.fire(&element, "onblur", Value::Null);
                                    }
                                }
                            }
                        }
                    }
                    InputEvent::MouseButtonUp(button) => {
                        if button == MouseButtons::Left {
                            element.state.active = false;
                            if element.state.hover {
                                match element.tag.as_str() {
                                    "option" => self.handle_option_click(node, tree)?,
                                    _ => {
                                        self.fire(&element, "onclick", Value::Null);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        for child in tree.children(node).expect("node {node:?} children must be") {
            self.capture_element_events(child, tree, events)?;
        }
        Ok(())
    }

    pub(crate) fn fire(&mut self, element: &Element, event: &str, this: Value) {
        if let Some(handler) = element.listeners.get(event) {
            let mut arguments = vec![];
            for argument in &handler.arguments {
                match argument {
                    CallbackArgument::This => {
                        arguments.push(this.clone());
                    }
                    CallbackArgument::Binder(path, pipe) => {
                        let mut value = self.model.pointer(path).cloned().unwrap_or(Value::Null);
                        for name in pipe {
                            match self.transformers.get(name) {
                                Some(transform) => value = transform(value),
                                None => {
                                    error!("unable to transform argument, transformer {name} not found")
                                }
                            }
                        }
                        arguments.push(value);
                    }
                }
            }
            self.output.calls.push(Call {
                function: handler.function.clone(),
                arguments,
            })
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
    Visibility(NodeId, NodeId, bool),
    Attribute(NodeId, String, usize),
    Tag(NodeId, String),
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
            BindingParams::Tag(node, key) => Reaction::Tag {
                node,
                key,
                tag: value.as_boolean(),
            },
            BindingParams::Attribute(node, key, span) => Reaction::Bind {
                node,
                key,
                span,
                text: value.eval_string(),
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

#[derive(Debug, PartialEq)]
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
    Tag {
        node: NodeId,
        key: String,
        tag: bool,
    },
    Bind {
        node: NodeId,
        key: String,
        span: usize,
        text: String,
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
#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    /// The identifier of event handler (function name probably).
    pub function: String,
    /// The JSON-like arguments.
    pub arguments: Vec<Value>,
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

#[derive(Debug, Clone)]
pub struct Handler {
    pub function: String,
    pub arguments: Vec<CallbackArgument>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CallbackArgument {
    This,
    Binder(String, Vec<String>),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_rebind_simple_array_same_values_reduced_array() {
        let model = json!({
            "names": [null, null, null]
        });
        let [names, names_0, names_1, names_2] = [100, 200, 300, 400];
        let bindings = BTreeMap::from([
            ("/names".to_string(), vec![repeat(names, 3)]),
            ("/names/0".to_string(), vec![text(names_0)]),
            ("/names/1".to_string(), vec![text(names_1)]),
            ("/names/2".to_string(), vec![text(names_2)]),
        ]);
        let mut view_model = ViewModel::create(bindings, model);
        view_model.bind(&json!({ "names": ["Alice", "Boris"] }));

        let reactions = view_model.bind(&json!({ "names": ["Alice"] }));

        assert_eq!(
            reactions,
            vec![Reaction::Repeat {
                parent: names.into(),
                start: 0,
                cursor: 1,
                end: 3,
            },]
        );
    }

    #[test]
    pub fn test_rebind_simple_array_new_values() {
        let model = json!({
            "names": [null, null, null]
        });
        let [names, names_0, names_1, names_2] = [100, 200, 300, 400];
        let bindings = BTreeMap::from([
            ("/names".to_string(), vec![repeat(names, 3)]),
            ("/names/0".to_string(), vec![text(names_0)]),
            ("/names/1".to_string(), vec![text(names_1)]),
            ("/names/2".to_string(), vec![text(names_2)]),
        ]);
        let mut view_model = ViewModel::create(bindings, model);
        view_model.bind(&json!({ "names": ["Alice", "Boris"] }));

        let reactions = view_model.bind(&json!({ "names": ["Carol", "David"] }));

        assert_eq!(
            reactions,
            vec![
                Reaction::Type {
                    node: names_0.into(),
                    span: 0,
                    text: "Carol".to_string(),
                },
                Reaction::Type {
                    node: names_1.into(),
                    span: 0,
                    text: "David".to_string()
                }
            ]
        );
    }

    #[test]
    pub fn test_rebind_simple_array_new_values_increased_array() {
        let model = json!({
            "names": [null, null, null]
        });
        let [names, names_0, names_1, names_2] = [100, 200, 300, 400];
        let bindings = BTreeMap::from([
            ("/names".to_string(), vec![repeat(names, 3)]),
            ("/names/0".to_string(), vec![text(names_0)]),
            ("/names/1".to_string(), vec![text(names_1)]),
            ("/names/2".to_string(), vec![text(names_2)]),
        ]);
        let mut view_model = ViewModel::create(bindings, model);
        view_model.bind(&json!({ "names": ["Alice"] }));

        let reactions = view_model.bind(&json!({ "names": ["Boris", "Carol"] }));

        assert_eq!(
            reactions,
            vec![
                Reaction::Repeat {
                    parent: names.into(),
                    start: 0,
                    cursor: 2,
                    end: 3,
                },
                Reaction::Type {
                    node: names_0.into(),
                    span: 0,
                    text: "Boris".to_string(),
                },
                Reaction::Type {
                    node: names_1.into(),
                    span: 0,
                    text: "Carol".to_string()
                }
            ]
        );
    }

    #[test]
    pub fn test_rebind_objects_array_same_values_reduced_array() {
        let model = json!({
            "items": [
                {"id": null, "name": null},
                {"id": null, "name": null},
                {"id": null, "name": null}
            ]
        });
        let [items, items_0, items_1, items_2] = [100, 200, 300, 400];
        let bindings = BTreeMap::from([
            ("/items".to_string(), vec![repeat(items, 3)]),
            ("/items/0/name".to_string(), vec![text(items_0)]),
            ("/items/0/id".to_string(), vec![attr(items_0, "id", 0)]),
            ("/items/1/name".to_string(), vec![text(items_1)]),
            ("/items/1/id".to_string(), vec![attr(items_1, "id", 0)]),
            ("/items/2/name".to_string(), vec![text(items_2)]),
            ("/items/2/id".to_string(), vec![attr(items_2, "id", 0)]),
        ]);
        let mut view_model = ViewModel::create(bindings, model);
        view_model.bind(&json!({
            "items": [
                {"id": 0, "name": "Alice"},
                {"id": 1, "name": "Boris"}
            ]
        }));

        let reactions = view_model.bind(&json!({
            "items": [
                {"id": 0, "name": "Alice"},
            ]
        }));

        assert_eq!(
            reactions,
            vec![Reaction::Repeat {
                parent: items.into(),
                start: 0,
                cursor: 1,
                end: 3,
            }]
        );
    }

    #[test]
    pub fn test_rebind_objects_array_new_values_increased_array() {
        let model = json!({
            "items": [
                {"id": null, "name": null},
                {"id": null, "name": null},
                {"id": null, "name": null}
            ]
        });
        let [items, items_0, items_1, items_2] = [100, 200, 300, 400];
        let bindings = BTreeMap::from([
            ("/items".to_string(), vec![repeat(items, 3)]),
            ("/items/0/name".to_string(), vec![text(items_0)]),
            ("/items/0/id".to_string(), vec![attr(items_0, "id", 0)]),
            ("/items/1/name".to_string(), vec![text(items_1)]),
            ("/items/1/id".to_string(), vec![attr(items_1, "id", 0)]),
            ("/items/2/name".to_string(), vec![text(items_2)]),
            ("/items/2/id".to_string(), vec![attr(items_2, "id", 0)]),
        ]);
        let mut view_model = ViewModel::create(bindings, model);
        view_model.bind(&json!({
            "items": [
                {"id": 0, "name": "Alice"}
            ]
        }));

        let reactions = view_model.bind(&json!({
            "items": [
                {"id": 1, "name": "Boris"},
                {"id": 2, "name": "Carol"},
            ]
        }));

        assert_eq!(
            reactions,
            vec![
                Reaction::Repeat {
                    parent: items.into(),
                    start: 0,
                    cursor: 2,
                    end: 3,
                },
                Reaction::Bind {
                    node: items_0.into(),
                    key: "id".to_string(),
                    span: 0,
                    text: "1".to_string(),
                },
                Reaction::Type {
                    node: items_0.into(),
                    span: 0,
                    text: "Boris".to_string(),
                },
                Reaction::Bind {
                    node: items_1.into(),
                    key: "id".to_string(),
                    span: 0,
                    text: "2".to_string(),
                },
                Reaction::Type {
                    node: items_1.into(),
                    span: 0,
                    text: "Carol".to_string()
                }
            ]
        );
    }

    fn node(id: u64) -> NodeId {
        NodeId::new(id)
    }

    fn repeat(node: u64, n: usize) -> Binding {
        Binding {
            params: BindingParams::Repeat(NodeId::new(node), 0, n),
            pipe: vec![],
        }
    }

    fn text(node: u64) -> Binding {
        Binding {
            params: BindingParams::Text(NodeId::new(node), 0),
            pipe: vec![],
        }
    }

    fn attr(node: u64, attr: &str, span: usize) -> Binding {
        Binding {
            params: BindingParams::Attribute(NodeId::new(node), attr.to_string(), span),
            pipe: vec![],
        }
    }
}
