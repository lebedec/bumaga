use crate::css::{match_style, Str};
use crate::html::{Binder, ElementBinding, Html, TextBinding, TextSpan};
use crate::input::FakeFonts;
use crate::models::{ElementId, Object, Sizes};
use crate::state::State;
use crate::styles::{create_element, default_layout, inherit, Cascade};
use crate::view_model::{Binding, Bindings, Reaction};
use crate::{
    CallOld, Component, ComponentError, Element, Fonts, Handler, Input, TextContent,
    ValueExtensions,
};
use log::error;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::fmt::format;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::mem;
use std::mem::take;
use std::process::id;
use std::time::Instant;
use taffy::prelude::length;
use taffy::style_helpers::TaffyMaxContent;
use taffy::{
    AlignItems, AvailableSpace, Dimension, Display, JustifyContent, NodeId, Size, Style,
    TaffyError, TaffyTree, TraversePartialTree,
};

pub struct Schema {
    pub value: Value,
}

pub enum Token {
    None,
    Field(String),
    Array(String, usize),
}

impl Schema {
    pub fn new() -> Self {
        Self { value: json!({}) }
    }

    pub fn index(&mut self, binder: &Binder, i: usize, locals: &HashMap<String, String>) -> String {
        let path = self.get_value_path(&binder.path, locals);
        let path = format!("{path}[{i}]");
        Self::define_value(&mut self.value, &mut format!("{path}."));
        path
    }

    pub fn field(&mut self, binder: &Binder, locals: &HashMap<String, String>) -> String {
        println!("FIELD {}", binder.to_string());
        let path = self.get_value_path(&binder.path, locals);
        Self::define_value(&mut self.value, &mut format!("{path}."));
        path
    }

    fn get_value_path(&mut self, path: &Vec<String>, locals: &HashMap<String, String>) -> String {
        let head = &path[0];
        let head = locals.get(head).unwrap_or(head);
        let tail = &path[1..];
        if tail.len() > 0 {
            format!("{head}.{}", tail.join("."))
        } else {
            format!("{head}")
        }
    }

    fn parse(input: &mut String) -> Token {
        let mut field = String::new();
        let mut index = String::new();
        while input.len() > 0 {
            let mut ch = input.remove(0);
            if ch == '.' {
                let field = take(&mut field);
                let index = take(&mut index);
                return if index.is_empty() {
                    Token::Field(field)
                } else {
                    Token::Array(field, index.parse().unwrap_or(0))
                };
            } else if ch == '[' {
                while input.len() > 0 {
                    let ch = input.remove(0);
                    if ch == ']' {
                        break;
                    } else {
                        index.push(ch);
                    }
                }
            } else {
                field.push(ch);
            }
        }
        Token::None
    }

    fn define_value(target: &mut Value, path: &mut String) {
        let token = Self::parse(path);
        match token {
            Token::None => {}
            Token::Field(field) => {
                if !target.is_object() {
                    *target = json!({});
                }
                let object = target.as_object_mut().unwrap();
                if !object.contains_key(&field) {
                    object.insert(field.clone(), Value::Null);
                }
                Self::define_value(object.get_mut(&field).unwrap(), path)
            }
            Token::Array(field, n) => {
                if !target.is_object() {
                    *target = json!({});
                }
                let object = target.as_object_mut().unwrap();
                if !object
                    .get(&field)
                    .map(|field| field.is_array())
                    .unwrap_or(false)
                {
                    object.insert(field.clone(), json!([]));
                }
                let array = object
                    .get_mut(&field)
                    .and_then(|field| field.as_array_mut())
                    .unwrap();
                if array.len() <= n {
                    array.resize(n + 1, Value::Null);
                }
                Self::define_value(&mut array[n], path)
            }
        }
    }
}

impl Component {
    pub fn render_tree(
        &mut self,
        input: &mut Input,
    ) -> Result<(NodeId, TaffyTree<Element>), ComponentError> {
        let mut rendering = TaffyTree::new();
        let [viewport_width, viewport_height] = input.viewport;
        let root_id = ElementId::fake();
        let root_layout = Style {
            size: Size {
                width: length(viewport_width),
                height: length(viewport_height),
            },
            ..Default::default()
        };
        let root = create_element(root_id, Object::tag(":root"));
        let context = Sizes {
            root_font_size: root.text_style.font_size,
            parent_font_size: root.text_style.font_size,
            viewport_width,
            viewport_height,
        };
        let root = rendering.new_leaf_with_context(root_layout, root)?;
        let html = self.html.content.clone();
        // TODO: determine body element
        let body = html.children.last().cloned().expect("body must be found");
        // self.state.active_animators = take(&mut self.state.animators);
        self.render_tree_node_old(root, body, input, context, &mut rendering);
        return Ok((root, rendering));
    }

    fn render_tree_node_old(
        &mut self,
        parent_id: NodeId,
        current: Html,
        input: &mut Input,
        context: Sizes,
        layout: &mut TaffyTree<Element>,
    ) {
        if current.text.is_some() {
            self.render_text_ol(parent_id, current, input, layout);
        } else {
            self.render_template(parent_id, current, input, context, layout)
        }
    }

    pub fn update_tree(&mut self, reactions: Vec<Reaction>) -> Result<(), ComponentError> {
        for reaction in reactions {
            match reaction {
                Reaction::Type { node, span, text } => {
                    let element_text = self
                        .tree
                        .get_node_context_mut(node)
                        .and_then(|element| element.text.as_mut())
                        .ok_or(ComponentError::ElementTextContentNotFound)?;
                    element_text.spans[span] = text;
                    self.tree.mark_dirty(node)?;
                }
                Reaction::Reattach { node, visible } => {
                    let parent = self
                        .tree
                        .parent(node)
                        .ok_or(ComponentError::ParentNotFound)?;
                    if visible {
                        self.tree.add_child(parent, node)?;
                    } else {
                        self.tree.remove_child(parent, node)?;
                    }
                }
                Reaction::Repeat {
                    parent,
                    start,
                    cursor,
                    end,
                } => {
                    let children = self
                        .tree
                        .get_node_context(parent)
                        .map(|element| element.children.clone())
                        .ok_or(ComponentError::ElementNotFound)?;
                    let shown = &children[start..cursor];
                    let hidden = &children[cursor..end];
                    for node in shown {
                        // The child is not removed from the tree entirely,
                        // it is simply no longer attached to its previous parent.
                        self.tree.remove_child(parent, *node)?;
                    }
                    for node in hidden {
                        self.tree.add_child(parent, *node)?;
                    }
                }
                Reaction::Bind { node, key, value } => {
                    let element = self
                        .tree
                        .get_node_context_mut(node)
                        .ok_or(ComponentError::ElementNotFound)?;
                    element.attrs.insert(key, value);
                }
            }
        }
        Ok(())
    }

    fn render_text_ol(
        &mut self,
        parent_id: NodeId,
        current: Html,
        input: &Input,
        layout: &mut TaffyTree<Element>,
    ) {
        let element_id = ElementId::from(&current);
        let text = current.text.unwrap_or_default();
        let text = interpolate_string(&text, input);
        let style = default_layout();
        let object = Object::text(text);
        let mut element = create_element(element_id, object);
        let parent = layout.get_node_context(parent_id).expect("context must be");
        inherit(&parent, &mut element);
        layout.new_child_of(parent_id, style, element);
    }

    pub fn render_template(
        &mut self,
        parent_id: NodeId,
        current: Html,
        input: &mut Input,
        context: Sizes,
        layout: &mut TaffyTree<Element>,
    ) {
        if let Some(pipe) = current.attrs.get("?") {
            if !is_something(Some(&get_object_value(pipe, input))) {
                return;
            }
        }
        if let Some(ident) = current.attrs.get("!") {
            if is_something(input.value.get(ident)) {
                return;
            }
        }
        let repetition = Repetition::from(&input.value, current.attrs.get("*"));
        for repeat_value in repetition.values {
            // PUSH STATE
            input
                .value
                .insert(repetition.key.to_string(), repeat_value.clone());
            // RENDER
            let element_id = ElementId::hash(&current, &repeat_value);
            self.render_element_old(element_id, parent_id, &current, input, context, layout);
            // POP STATE
            input.value.remove(&repetition.key);
        }
    }

    pub fn render_node(
        template: Html,
        tree: &mut TaffyTree<Element>,
        bindings: &mut Bindings,
        locals: &mut HashMap<String, String>,
        schema: &mut Schema,
    ) -> Result<NodeId, ComponentError> {
        if let Some(text) = template.text_new {
            Self::render_text(text, tree, bindings, locals, schema)
        } else {
            Self::render_template2(template, tree, bindings, locals, schema)
        }
    }

    pub fn render_text(
        text: TextBinding,
        tree: &mut TaffyTree<Element>,
        bindings: &mut Bindings,
        locals: &mut HashMap<String, String>,
        schema: &mut Schema,
    ) -> Result<NodeId, ComponentError> {
        let layout = default_layout();
        let node = tree.new_leaf(layout)?;
        let spans = text
            .spans
            .into_iter()
            .enumerate()
            .map(|(index, span)| match span {
                TextSpan::String(span) => span,
                TextSpan::Binder(binder) => {
                    let path = schema.field(&binder, locals);
                    let binding = Binding::Text(node, index);
                    bindings.entry(path).or_default().push(binding);
                    binder.to_string()
                }
            })
            .collect();
        let mut element = create_element(ElementId::fake(), Object::fake());
        element.text = Some(TextContent { spans });
        tree.set_node_context(node, Some(element))?;
        Ok(node)
    }

    pub fn render_template2(
        template: Html,
        tree: &mut TaffyTree<Element>,
        bindings: &mut Bindings,
        locals: &mut HashMap<String, String>,
        schema: &mut Schema,
    ) -> Result<NodeId, ComponentError> {
        let mut overridden = HashMap::new();
        for binding in &template.bindings {
            if let ElementBinding::Alias(name, binder) = binding {
                let path = schema.field(binder, locals);
                overridden.insert(name.to_string(), locals.insert(name.to_string(), path));
            }
        }
        let node = Self::render_element(template, tree, bindings, locals, schema)?;
        for (key, value) in overridden {
            if let Some(value) = value {
                locals.insert(key, value);
            } else {
                locals.remove(&key);
            }
        }
        Ok(node)
    }

    pub fn render_element(
        template: Html,
        tree: &mut TaffyTree<Element>,
        bindings: &mut Bindings,
        locals: &mut HashMap<String, String>,
        schema: &mut Schema,
    ) -> Result<NodeId, ComponentError> {
        let layout = default_layout();
        let node = tree.new_leaf(layout)?;
        let mut element = create_element(ElementId::fake(), Object::fake());
        element.tag = template.tag.clone();
        for binding in template.bindings {
            match binding {
                ElementBinding::None(key, value) => {
                    element.attrs.insert(key, value);
                }
                ElementBinding::Attribute(key, binder) => {
                    let path = schema.field(&binder, locals);
                    let binding = Binding::Attribute(node, key.clone());
                    bindings.entry(path).or_default().push(binding);
                    element.attrs.insert(key, binder.to_string());
                }
                ElementBinding::Callback(event, function, argument) => {
                    let handler = Handler { function, argument };
                    element.listeners.insert(event.clone(), handler);
                }
                ElementBinding::Visibility(visible, binder) => {
                    let path = schema.field(&binder, locals);
                    let binding = Binding::Visibility(node, visible);
                    bindings.entry(path).or_default().push(binding);
                }
                _ => {}
            }
        }
        let mut children = vec![];
        for child in template.children {
            if let Some((name, count, binder)) = child.as_repeat() {
                let array = schema.field(binder, locals);
                let start = children.len();
                let binding = Binding::Repeat(node, start, count);
                bindings.entry(array.clone()).or_default().push(binding);
                let overridden = locals.remove(name);
                for n in 0..count {
                    let path = schema.index(binder, n, locals);
                    locals.insert(name.to_string(), path);
                    let child = child.clone();
                    let child = Self::render_node(child, tree, bindings, locals, schema)?;
                    children.push(child);
                }
                if let Some(overridden) = overridden {
                    locals.insert(name.to_string(), overridden);
                } else {
                    locals.remove(name);
                }
            } else {
                let child = Self::render_node(child, tree, bindings, locals, schema)?;
                children.push(child);
            }
        }
        tree.set_node_context(node, Some(element))?;
        tree.set_children(node, &children)?;
        Ok(node)
    }

    fn render_element_old(
        &mut self,
        element_id: ElementId,
        parent_id: NodeId,
        template: &Html,
        input: &mut Input,
        sizes: Sizes,
        tree: &mut TaffyTree<Element>,
    ) {
        let current_id = tree.new_leaf(default_layout()).expect("id must be created");
        tree.add_child(parent_id, current_id)
            .expect("child must be added");

        let mut current = template.clone();
        for (key, pipe) in &template.attrs {
            if key.starts_with("data-") || key.starts_with("value") {
                let string = get_object_value(&pipe, input).as_string();
                current.attrs.insert(key.clone(), string.into());
            }
        }

        let object = Object::element(&current);
        let mut element = create_element(element_id, object);
        self.state.restore(&mut element);

        // APPLY STYLES
        let mut layout = default_layout();
        // preset element context for CSS matching process
        tree.set_node_context(current_id, Some(element.clone()))
            .expect("context must be set");
        let parent = tree.get_node_context(parent_id).expect("context must be");
        // default html styles
        match element.html.tag.as_str() {
            "input" => {
                layout.display = Display::Flex;
                layout.align_items = Some(AlignItems::Center);
            }
            _ => {}
        }

        let mut cascade = Cascade::new(&self.css.content, sizes, &self.resources);
        cascade.apply_styles(input, current_id, tree, parent, &mut layout, &mut element);

        self.render_output_bindings(&mut element, &input);

        // final commit of style changes
        tree.set_style(current_id, layout)
            .expect("style must be updated");
        tree.set_node_context(current_id, Some(element.clone()))
            .expect("context must be set");

        match element.html.tag.as_str() {
            "img" => {
                self.render_img(current_id, &element, tree);
            }
            "input" => {
                let text = element.html.attrs.get("value").cloned().unwrap_or_default();
                self.render_input(text, current_id, &element, tree);
            }
            "area" => {}
            "base" => {}
            "br" => {}
            "col" => {}
            "command" => {}
            "embed" => {}
            "hr" => {}
            "keygen" => {}
            "link" => {}
            "meta" => {}
            "param" => {}
            "source" => {}
            "track" => {}
            "wbr" => {}
            _ => {
                for child in template.children.clone() {
                    let mut context = sizes;
                    context.parent_font_size = element.text_style.font_size;
                    self.render_tree_node_old(current_id, child, input, context, tree);
                }
            }
        }
    }

    fn render_img(&mut self, parent_id: NodeId, parent: &Element, layout: &mut TaffyTree<Element>) {
        let element_id = ElementId::fake();
        let empty = "undefined.png".to_string();
        let src = parent.html.attrs.get("src").unwrap_or(&empty);
        let src = format!("{}{}", self.resources, src);
        let object = Object::fake();
        let mut element = create_element(element_id, object);
        element.background.image = Some(src);
        let style = Style {
            size: Size {
                width: Dimension::Percent(1.0),
                height: Dimension::Percent(1.0),
            },
            ..default_layout()
        };
        layout.new_child_of(parent_id, style, element);
    }

    fn render_input(
        &mut self,
        text: String,
        parent_id: NodeId,
        parent: &Element,
        layout: &mut TaffyTree<Element>,
    ) {
        let element_id = ElementId::child(parent.id, 1);
        let style = default_layout();
        let object = Object::text(text);
        let mut element = create_element(element_id, object);
        inherit(&parent, &mut element);
        layout.new_child_of(parent_id, style, element);

        let element_id = ElementId::child(parent.id, 2);
        if parent.html.pseudo_classes.contains("focus") {
            let object = Object::fake();
            let mut element = create_element(element_id, object);
            let mut style = default_layout();
            style.size.width = Dimension::Length(1.0);
            style.size.height = Dimension::Length(element.text_style.font_size);
            element.background.color = element.color;
            layout.new_child_of(parent_id, style, element);
        }
    }

    /// NOTE: must be in rendering cycle because scope contains repeated values
    /// TODO: analyze performance issues (skip call render if no events)
    /// Configures the elements or adjust their behavior in various ways to meet HTML experience.
    ///
    /// see details: https://developer.mozilla.org/en-US/docs/Web/HTML/Attributes
    fn render_output_bindings(&self, element: &mut Element, input: &Input) {
        let events: &[&str] = match element.html.tag.as_ref() {
            "input" => &["oninput", "onchange"],
            _ => &["onclick"],
        };
        for event in events {
            if let Some(expr) = element.html.attrs.get(*event) {
                let output = eval_call(expr, input);
                element.listeners_old.insert(event.to_string(), output);
            }
        }
    }
}

trait TaffyTreeExtensions {
    fn new_child_of(&mut self, parent_id: NodeId, style: Style, element: Element)
        -> Option<NodeId>;
}

impl TaffyTreeExtensions for TaffyTree<Element> {
    fn new_child_of(
        &mut self,
        parent_id: NodeId,
        style: Style,
        element: Element,
    ) -> Option<NodeId> {
        let node_id = match self.new_leaf_with_context(style, element) {
            Ok(node_id) => node_id,
            Err(error) => {
                error!("unable to create child node, {}", error);
                return None;
            }
        };
        if let Err(error) = self.add_child(parent_id, node_id) {
            error!("unable to add child node, {}", error);
            return None;
        }
        Some(node_id)
    }
}

pub fn is_something(value: Option<&serde_json::Value>) -> bool {
    match value {
        None => false,
        Some(value) => match value {
            Value::Null => false,
            serde_json::Value::Bool(value) => *value,
            serde_json::Value::Number(number) => number.as_f64() != Some(0.0),
            serde_json::Value::String(value) => !value.is_empty(),
            serde_json::Value::Array(value) => !value.is_empty(),
            serde_json::Value::Object(_) => true,
        },
    }
}

pub fn as_array(value: Option<&Value>) -> Option<&Vec<Value>> {
    match value {
        None => None,
        Some(value) => value.as_array(),
    }
}

pub fn as_string(value: Option<&Value>) -> String {
    match value {
        None => String::new(),
        Some(value) => match value {
            Value::Null => String::new(),
            Value::Bool(value) => value.to_string(),
            Value::Number(value) => value.to_string(),
            Value::String(value) => value.clone(),
            Value::Array(_) => String::from("[array]"),
            Value::Object(_) => String::from("[object]"),
        },
    }
}

fn integer_decode(val: f64) -> (u64, i16, i8) {
    let bits: u64 = unsafe { mem::transmute(val) };
    let sign: i8 = if bits >> 63 == 0 { 1 } else { -1 };
    let mut exponent: i16 = ((bits >> 52) & 0x7ff) as i16;
    let mantissa = if exponent == 0 {
        (bits & 0xfffffffffffff) << 1
    } else {
        (bits & 0xfffffffffffff) | 0x10000000000000
    };
    exponent -= 1023 + 52;
    (mantissa, exponent, sign)
}

pub fn get_object_value(pipe: &str, input: &Input) -> serde_json::Value {
    let mut value = serde_json::Value::Null;
    let segments: Vec<&str> = pipe.split("|").map(&str::trim).collect();
    let getters = match segments.get(0) {
        None => {
            error!("empty pipe");
            return value;
        }
        Some(path) => {
            let getters: Vec<String> = path
                .split(".")
                .map(|getter| getter.trim().to_string())
                .collect();
            if getters.len() == 0 {
                error!("empty getters");
                return value;
            }
            getters
        }
    };
    let mut scope = &input.value;
    for i in 0..getters.len() - 1 {
        let getter = &getters[i];
        scope = match scope.get(getter).and_then(|v| v.as_object()) {
            None => {
                error!("nested attribute '{getter}' not object");
                return value;
            }
            Some(nested) => nested,
        }
    }
    let attr = &getters[getters.len() - 1];
    value = match scope.get(attr) {
        None => {
            error!("attribute '{attr}' not found");
            return value;
        }
        Some(value) => value.clone(),
    };
    for name in segments.iter().skip(1) {
        match input.transformers.get(*name) {
            None => error!("transformer {name} not registered"),
            Some(transform) => value = transform(value),
        }
    }
    value
}

pub fn interpolate_string(string: &str, input: &Input) -> String {
    let mut result = String::new();
    let mut field = false;
    let mut pipe = String::new();
    for ch in string.chars() {
        if field {
            if ch == '}' {
                result += &get_object_value(&pipe, input).as_string();
                field = false;
            } else {
                pipe.push(ch);
            }
        } else {
            if ch == '{' {
                field = true;
                pipe = String::new();
            }
            if !field {
                result.push(ch);
            }
        }
    }
    result
}

fn eval_call(expression: &str, input: &Input) -> CallOld {
    let mut function = String::new();
    let mut arguments = vec![];
    let mut is_function = true;
    let mut arg = String::new();
    for ch in expression.chars() {
        if is_function {
            if ch == '(' {
                is_function = false;
            } else {
                function.push(ch);
            }
        } else {
            if ch == ',' || ch == ')' {
                let value = arg.trim().replace("'", "\"");
                let value: Value = match serde_json::from_str(&value) {
                    Ok(value) => value,
                    Err(_) => input.value.get(&value).cloned().unwrap_or(Value::Null),
                };
                arguments.push(value);
                arg = String::new();
            } else {
                arg.push(ch);
            }
        }
    }
    CallOld {
        function,
        arguments,
    }
}

struct Repetition {
    key: String,
    values: Vec<Value>,
}

impl Repetition {
    pub fn from(globals: &Map<String, Value>, key: Option<&String>) -> Self {
        let key = match key {
            None => return Repetition::no(),
            Some(key) => key,
        };
        match as_array(globals.get(key)) {
            None => {
                error!("unable to repeat {key}, it must be JSON array");
                Self::no()
            }
            Some(values) => Self {
                key: key.to_string(),
                values: values.clone(),
            },
        }
    }

    pub fn no() -> Self {
        Self {
            key: "".to_string(),
            values: vec![Value::Null],
        }
    }
}
