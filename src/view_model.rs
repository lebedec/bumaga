use crate::{
    ControlEvent, ControlTarget, Element, Input, InputEvent, MouseButtons, Output, PointerEvents,
    ValueExtensions, View, ViewError, ViewResponse,
};
use log::error;

use crate::tree::ViewTreeExtensions;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::mem::{forget, take};
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
            mouse_hovers: HashSet::new(),
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
        for event in input.events.iter() {
            match *event {
                InputEvent::MouseMove(mouse) => {
                    self.mouse = mouse;
                }
                _ => {}
            }
        }
        self.output = Output::new();
        self.capture_element_input(body, tree, &input.events)?;
        self.output.is_input_captured =
            !self.mouse_hovers.is_empty() || self.drag.is_some() || self.focus.is_some();
        Ok(take(&mut self.output))
    }

    fn capture_element_input(
        &mut self,
        node: NodeId,
        tree: &mut TaffyTree<Element>,
        events: &Vec<InputEvent>,
    ) -> Result<(), ViewError> {
        let mut element = tree.get_element_mut(node)?;
        for input_event in events.iter().copied() {
            if element.state.focus {
                match input_event {
                    InputEvent::Char(char) => {
                        self.control(element, "oninput", ControlEvent::OnInput(char))
                    }
                    InputEvent::KeyDown(key) => {
                        self.control(element, "onkeydown", ControlEvent::OnKeyDown(key))
                    }
                    InputEvent::KeyUp(key) => {
                        self.control(element, "onkeyup", ControlEvent::OnKeyUp(key))
                    }
                    _ => {}
                }
            }
            if element.pointer_events == PointerEvents::Auto {
                match input_event {
                    InputEvent::MouseWheel(wheel) if element.state.hover => {
                        if let Some(scrolling) = element.scrolling.as_mut() {
                            scrolling.offset(wheel);
                        }
                    }
                    InputEvent::MouseButtonDown(button) => {
                        if element.state.hover {
                            self.control(&element, "onmousedown", ControlEvent::OnMouseDown);
                            if button == MouseButtons::Left {
                                element.state.active = true;
                            }
                            if !element.state.focus {
                                element.state.focus = true;
                                self.focus = Some(node);
                                self.control(&element, "onfocus", ControlEvent::OnFocus);
                            }
                            if button == MouseButtons::Left && element.draggable() {
                                self.control(element, "ondragstart", ControlEvent::OnDragStart);
                                self.drag = DragContext::new(node);
                            }
                        } else {
                            if element.state.focus {
                                if self.focus == Some(node) {
                                    // reset focus only if not reassigned by other element
                                    self.focus = None;
                                }
                                element.state.focus = false;
                                self.control(&element, "onblur", ControlEvent::OnBlur);
                            }
                        }
                    }
                    InputEvent::MouseButtonUp(button) => {
                        if element.state.hover {
                            self.control(&element, "onmouseup", ControlEvent::OnMouseUp);
                            if let Some(drag) = self.drag.as_mut() {
                                if element.listeners.contains_key("ondrop") {
                                    // valid drop target
                                    let source = drag.source;
                                    self.control(element, "ondrop", ControlEvent::OnDrop);
                                    self.drag = None;
                                    element = tree.get_element_mut(source)?;
                                    self.control(element, "ondragend", ControlEvent::OnDragEnd);
                                    element = tree.get_element_mut(node)?;
                                }
                            } else {
                                if button == MouseButtons::Left && element.state.active {
                                    self.control(&element, "onclick", ControlEvent::OnClick);
                                }
                                if button == MouseButtons::Right {
                                    self.control(
                                        &element,
                                        "oncontextmenu",
                                        ControlEvent::OnContextMenu,
                                    );
                                }
                            }
                        }
                        element.state.active = false;
                    }
                    InputEvent::MouseMove(m) => {
                        let hover = hovers(self.mouse, element);
                        if hover {
                            if !element.state.hover {
                                element.state.hover = true;
                                self.control(element, "onmouseenter", ControlEvent::OnMouseEnter);
                                self.mouse_hovers.insert(element.node);
                                if self.drag.is_some() {
                                    self.control(element, "ondragenter", ControlEvent::OnDragEnter);
                                }
                            }
                        } else {
                            if element.state.hover {
                                element.state.hover = false;
                                self.control(element, "onmouseleave", ControlEvent::OnMouseLeave);
                                self.mouse_hovers.remove(&element.node);
                                if self.drag.is_some() {
                                    self.control(element, "ondragleave", ControlEvent::OnDragLeave);
                                }
                            }
                        }
                        if element.state.hover {
                            self.control(element, "onmousemove", ControlEvent::OnMouseMove);
                            if self.drag.is_some() {
                                self.control(element, "ondragover", ControlEvent::OnDragOver);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        for child in tree.children(node).expect("node {node:?} children must be") {
            self.capture_element_input(child, tree, events)?;
        }
        Ok(())
    }

    pub(crate) fn control(&mut self, element: &Element, handler: &str, event: ControlEvent) {
        if let Some(path) = element.listeners.get(handler) {
            if let Some(id) = self.model.pointer(path).and_then(|id| id.as_u64()) {
                let target = ControlTarget::create(element, self.mouse);
                self.output.responses.push(ViewResponse {
                    id: id as usize,
                    event,
                    target,
                })
            }
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
