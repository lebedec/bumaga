use crate::{
    Element, ElementState, HandlerArgument, Input, InputEvent, Keys, MouseButtons, Output,
    PointerEvents, ValueExtensions, ViewError,
};
use log::error;

use crate::tree::ViewTreeExtensions;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, HashMap};
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
    pub(crate) elements_under_mouse: Vec<NodeId>,
    pub(crate) elements_in_action: Vec<NodeId>,
    output: Output,
    pub(crate) drag: Option<DragContext>,
    pub(crate) focus: Option<NodeId>,
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
            elements_under_mouse: Vec::new(),
            elements_in_action: vec![],
            output: Output::new(),
            drag: None,
            focus: None,
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
    ) -> bool {
        match (&mut dst, src) {
            (Value::Array(current), Value::Array(next)) => {
                let mut array_changed = false;
                if current.len() != next.len() {
                    if let Some(default) = default.get(arrays_path).cloned() {
                        array_changed = true;
                        current.resize(next.len(), default);
                        Self::react(path, src, bindings, reactions, transformers);
                    } else {
                        error!("unable to resize array {path} default not found");
                    }
                }
                for (index, dst) in current.iter_mut().enumerate() {
                    let src = &next[index];
                    let path = format!("{path}/{index}");
                    let changed = Self::bind_value(
                        dst,
                        src,
                        &path,
                        arrays_path,
                        bindings,
                        reactions,
                        transformers,
                        default,
                    );
                    array_changed = array_changed || changed;
                }
                array_changed
            }
            (Value::Array(current), Value::Null) => {
                if current.len() != 0 {
                    current.clear();
                    Self::react(
                        path,
                        &Value::Array(vec![]),
                        bindings,
                        reactions,
                        transformers,
                    );
                    true
                } else {
                    false
                }
            }
            (Value::Array(_), _) => {
                error!("unable to bind '{path}', must be array");
                false
            }
            (Value::Object(object), Value::Object(src)) => {
                let mut object_changed = false;
                for (key, dst) in object.iter_mut() {
                    let path = format!("{path}/{key}");
                    let arrays_path = format!("{arrays_path}/{key}");
                    let undefined = Value::Null;
                    let src = match src.get(key) {
                        Some(src) => src,
                        None => {
                            // error!("unable to bind '{path}', must be specified");
                            &undefined
                        }
                    };
                    let changed = Self::bind_value(
                        dst,
                        src,
                        &path,
                        &arrays_path,
                        bindings,
                        reactions,
                        transformers,
                        default,
                    );
                    object_changed = object_changed || changed;
                }
                if object_changed {
                    Self::react(path, &json!({}), bindings, reactions, transformers);
                }
                object_changed
            }
            (Value::Object(object), Value::Null) => {
                let mut object_changed = false;
                for (key, dst) in object.iter_mut() {
                    let path = format!("{path}/{key}");
                    let arrays_path = format!("{arrays_path}/{key}");
                    let changed = Self::bind_value(
                        dst,
                        &Value::Null,
                        &path,
                        &arrays_path,
                        bindings,
                        reactions,
                        transformers,
                        default,
                    );
                    object_changed = object_changed || changed;
                }
                if object_changed {
                    Self::react(path, &Value::Null, bindings, reactions, transformers);
                }
                object_changed
            }
            (dst, src) => {
                if *dst != src {
                    **dst = src.clone();
                    Self::react(path, src, bindings, reactions, transformers);
                    true
                } else {
                    false
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
        let mut has_mouse_move = false;
        let mut events = input.events.clone();
        for event in events.iter() {
            match *event {
                InputEvent::MouseMove(mouse) => {
                    self.mouse = mouse;
                    has_mouse_move = true;
                }
                _ => {}
            }
        }
        if !has_mouse_move {
            // fake event to recalculate hovers event user not move mouse
            // need because CSS animation can change elements size and we need handle this
            // TODO: proper solution to fix problem
            events.insert(0, InputEvent::MouseMove(self.mouse))
        }
        self.output = Output::new();
        self.handle_elements_input(events, body, tree)?;
        self.output.is_input_captured = !self.elements_under_mouse.is_empty()
            || self.drag.is_some()
            || self.focus.is_some()
            || !self.elements_in_action.is_empty();
        Ok(take(&mut self.output))
    }

    fn handle_elements_input(
        &mut self,
        events: Vec<InputEvent>,
        body: NodeId,
        tree: &mut TaffyTree<Element>,
    ) -> Result<(), ViewError> {
        for event in events {
            match event {
                InputEvent::Unknown => {}
                InputEvent::MouseMove(position) => {
                    let previous_update = take(&mut self.elements_under_mouse);
                    self.calculate_mouse_hovers(tree, body, position)?;
                    for node in previous_update.iter().rev() {
                        if !self.elements_under_mouse.contains(node) {
                            let element = tree.get_element_mut(*node)?;
                            element.state.hover = false;
                            let event = MouseEvent::new(self.mouse, element);
                            self.emit(element, "onmouseleave", event);
                            if self.drag.is_some() {
                                let event = MouseEvent::new(self.mouse, element);
                                self.emit(element, "ondragleave", event);
                            }
                        }
                    }
                    let current = self.elements_under_mouse.clone();
                    for node in current.iter().rev() {
                        if !previous_update.contains(node) {
                            let element = tree.get_element_mut(*node)?;
                            element.state.hover = true;
                            let event = MouseEvent::new(self.mouse, element);
                            self.emit(element, "onmouseenter", event);
                            if self.drag.is_some() {
                                let event = MouseEvent::new(self.mouse, element);
                                self.emit(element, "ondragenter", event);
                            }
                        }
                        let element = tree.get_element_mut(*node)?;
                        let event = MouseEvent::new(self.mouse, element);
                        self.emit(element, "onmousemove", event);
                        if self.drag.is_some() {
                            let event = MouseEvent::new(self.mouse, element);
                            self.emit(element, "ondragover", event);
                        }
                    }
                }
                InputEvent::MouseButtonDown(button) => {
                    if let Some(focus) = self.focus {
                        if !self.elements_under_mouse.contains(&focus) {
                            self.focus = None;
                            let element = tree.get_element_mut(focus)?;
                            element.state.focus = false;
                            let event = MouseEvent::new(self.mouse, element);
                            self.emit(&element, "onblur", event);
                        }
                    }
                    let elements_under_mouse = self.elements_under_mouse.clone();
                    for node in elements_under_mouse.iter().copied().rev() {
                        let mut element = tree.get_element_mut(node)?;

                        element.state.active = true;
                        self.elements_in_action.push(node);

                        if element.listeners.contains_key("oninput") {
                            // valid focus target
                            if let Some(focus) = self.focus {
                                if focus != node {
                                    self.focus = None;
                                    element = tree.get_element_mut(focus)?;
                                    element.state.focus = false;
                                    let event = MouseEvent::new(self.mouse, element);
                                    self.emit(&element, "onblur", event);
                                    element = tree.get_element_mut(node)?;
                                }
                            }
                            if Some(node) != self.focus {
                                self.focus = Some(node);
                                element.state.focus = true;
                                let event = MouseEvent::new(self.mouse, element);
                                self.emit(&element, "onfocus", event);
                            }
                        }

                        let event = MouseEvent::new(self.mouse, element);
                        self.emit(&element, "onmousedown", event);
                        if button == MouseButtons::Left && element.draggable() {
                            let event = MouseEvent::new(self.mouse, element);
                            self.emit(element, "ondragstart", event);
                            self.drag = DragContext::new(node);
                        }
                    }
                }
                InputEvent::MouseButtonUp(button) => {
                    let elements_under_mouse = self.elements_under_mouse.clone();
                    for node in elements_under_mouse.iter().rev() {
                        let element = tree.get_element_mut(*node)?;
                        let event = MouseEvent::new(self.mouse, element);
                        self.emit(&element, "onmouseup", event);
                        if let Some(drag) = self.drag.as_mut() {
                            if element.listeners.contains_key("ondrop") {
                                // valid drop target
                                let source = drag.source;
                                let event = MouseEvent::new(self.mouse, element);
                                self.emit(element, "ondrop", event);
                                self.drag = None;
                                let element = tree.get_element_mut(source)?;
                                let event = MouseEvent::new(self.mouse, element);
                                self.emit(element, "ondragend", event);
                            }
                        } else {
                            if button == MouseButtons::Left && element.state.active {
                                let event = MouseEvent::new(self.mouse, element);
                                self.emit(&element, "onclick", event);
                            }
                            if button == MouseButtons::Right {
                                let event = MouseEvent::new(self.mouse, element);
                                self.emit(&element, "oncontextmenu", event);
                            }
                        }
                    }
                    for node in take(&mut self.elements_in_action) {
                        let element = tree.get_element_mut(node)?;
                        element.state.active = false;
                    }
                }
                InputEvent::MouseWheel(_) => {}
                InputEvent::KeyDown(key) => {
                    if let Some(node) = self.focus {
                        let element = tree.get_element(node)?;
                        let event = KeyboardEvent::new(key, element);
                        self.emit(element, "onkeydown", event)
                    }
                }
                InputEvent::KeyUp(key) => {
                    if let Some(node) = self.focus {
                        let element = tree.get_element(node)?;
                        let event = KeyboardEvent::new(key, element);
                        self.emit(element, "onkeyup", event)
                    }
                }
                InputEvent::Char(char) => {
                    if let Some(node) = self.focus {
                        let element = tree.get_element(node)?;
                        let event = TextEvent::new(char, element);
                        self.emit(element, "oninput", event)
                    }
                }
            }
        }
        Ok(())
    }

    fn calculate_mouse_hovers(
        &mut self,
        tree: &TaffyTree<Element>,
        node: NodeId,
        position: [f32; 2],
    ) -> Result<(), ViewError> {
        let element = tree.get_element(node)?;
        if element.pointer_events == PointerEvents::Auto
            && element.visible
            && hovers(position, &element)
        {
            self.elements_under_mouse.push(node);
        }
        for child in tree.children(node)? {
            self.calculate_mouse_hovers(tree, child, position)?;
        }
        Ok(())
    }

    pub(crate) fn emit<T: Serialize>(&mut self, element: &Element, handler: &str, event: T) {
        if let Some(handler) = element.listeners.get(handler) {
            let mut key = "Undefined".to_string();
            let mut arguments = vec![];
            for (index, argument) in handler.arguments.iter().enumerate() {
                let argument = match argument {
                    HandlerArgument::Keyword(keyword) => Value::String(keyword.clone()),
                    HandlerArgument::Event => match serde_json::to_value(&event) {
                        Ok(event) => event,
                        Err(error) => {
                            error!("unable to serialize event, {error:?}");
                            continue;
                        }
                    },
                    HandlerArgument::Binder { path, pipe } => {
                        let mut value = match self.model.pointer(&path).cloned() {
                            Some(value) => value,
                            None => {
                                error!("unable to get value at {path:?}, not found");
                                continue;
                            }
                        };
                        for name in pipe {
                            match self.transformers.get(name) {
                                Some(transform) => value = transform(value),
                                None => {
                                    error!("unable to get value {path:?}, transformer {name} not found");
                                    continue;
                                }
                            }
                        }
                        value
                    }
                };
                if index == 0 {
                    key = argument.eval_string();
                } else {
                    arguments.push(argument);
                }
            }
            let message = match arguments.len() {
                0 => Value::String(key),
                1 => {
                    let mut object = Map::new();
                    object.insert(key, arguments.into_iter().next().expect("one argument"));
                    Value::Object(object)
                }
                _ => {
                    let mut object = Map::new();
                    object.insert(key, Value::Array(arguments));
                    Value::Object(object)
                }
            };
            self.output.messages.push(message);
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct KeyboardEvent {
    pub key: Keys,
    pub target: EventTarget,
}

impl KeyboardEvent {
    pub fn new(key: Keys, element: &Element) -> Self {
        Self {
            key,
            target: EventTarget::create(element),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MouseEvent {
    pub position: [f32; 2],
    pub target: EventTarget,
}

impl MouseEvent {
    pub fn new(position: [f32; 2], element: &Element) -> Self {
        Self {
            position,
            target: EventTarget::create(element),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TextEvent {
    pub char: char,
    pub target: EventTarget,
}

impl TextEvent {
    pub fn new(char: char, element: &Element) -> Self {
        Self {
            char,
            target: EventTarget::create(element),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EventTarget {
    pub size: [f32; 2],
    pub position: [f32; 2],
    pub state: ElementState,
}

impl EventTarget {
    pub fn create(element: &Element) -> Self {
        Self {
            size: element.size,
            position: element.position,
            state: element.state,
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
                let visible = value.eval_boolean() == visible;
                Reaction::Reattach {
                    parent,
                    node,
                    visible,
                }
            }
            BindingParams::Tag(node, key) => Reaction::Tag {
                node,
                key,
                tag: value.eval_boolean(),
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
                        let slice = &value[size..count.min(size + 3)];
                        error!("unable to repeat all items of {parent:?}, expect {size} but {count}, drop {slice:?} ...");
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

#[derive(Debug)]
pub struct DragContext {
    source: NodeId,
}

impl DragContext {
    pub fn new(node: NodeId) -> Option<Self> {
        Some(Self { source: node })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_rebind_root_property_with_undefined() {
        let model = json!({
            "name": null,
            "description": null
        });
        let [name, desc] = [100, 200];
        let bindings = BTreeMap::from([
            ("/name".to_string(), vec![text(name)]),
            ("/description".to_string(), vec![text(desc)]),
        ]);
        let mut view_model = ViewModel::create(bindings, model);
        view_model.bind(&json!({
            "name": "Name",
            "description": "Description...",
        }));
        let reactions = view_model.bind(&json!({
            "name": "Alice",
        }));
        assert_eq!(
            reactions,
            vec![
                Reaction::Type {
                    node: desc.into(),
                    span: 0,
                    text: "".to_string(),
                },
                Reaction::Type {
                    node: name.into(),
                    span: 0,
                    text: "Alice".to_string(),
                },
            ]
        );
    }

    #[test]
    pub fn test_rebind_object_property_with_undefined() {
        let model = json!({
            "object": {
                "name": null,
                "description": null
            }
        });
        let [name, desc] = [100, 200];
        let bindings = BTreeMap::from([
            ("/object/name".to_string(), vec![text(name)]),
            ("/object/description".to_string(), vec![text(desc)]),
        ]);
        let mut view_model = ViewModel::create(bindings, model);
        view_model.bind(&json!({
            "object": {
                "name": "Name",
                "description": "Description...",
            }
        }));
        let reactions = view_model.bind(&json!({
            "object": {
                "name": "Alice",
            }
        }));
        assert_eq!(
            reactions,
            vec![
                Reaction::Type {
                    node: desc.into(),
                    span: 0,
                    text: "".to_string(),
                },
                Reaction::Type {
                    node: name.into(),
                    span: 0,
                    text: "Alice".to_string(),
                },
            ]
        );
    }

    #[test]
    pub fn test_rebind_object_with_null() {
        let model = json!({
            "tooltip": {
                "name": null,
                "description": null
            }
        });
        let [parent, node, name, desc] = [100, 200, 300, 400];
        let bindings = BTreeMap::from([
            ("/tooltip".to_string(), vec![cond_if(parent, node)]),
            ("/tooltip/name".to_string(), vec![text(name)]),
            ("/tooltip/description".to_string(), vec![text(desc)]),
        ]);
        let mut view_model = ViewModel::create(bindings, model);
        view_model.bind(&json!({
            "tooltip": {
                "name": "Name",
                "description": "Description...",
            }
        }));
        let reactions = view_model.bind(&json!({
            "tooltip": null
        }));
        assert_eq!(
            reactions,
            vec![
                Reaction::Type {
                    node: desc.into(),
                    span: 0,
                    text: "".to_string(),
                },
                Reaction::Type {
                    node: name.into(),
                    span: 0,
                    text: "".to_string(),
                },
                Reaction::Reattach {
                    parent: parent.into(),
                    node: node.into(),
                    visible: false,
                }
            ]
        );
    }

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

    fn cond_if(parent: u64, node: u64) -> Binding {
        Binding {
            params: BindingParams::Visibility(NodeId::new(parent), NodeId::new(node), true),
            pipe: vec![],
        }
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
