use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::mem;
use std::mem::take;
use std::process::id;

use crate::css::{match_style, CssSpan, CssValue};
use crate::html::Html;
use crate::input::FakeFonts;
use crate::models::{ElementId, Object, Sizes};
use crate::state::State;
use crate::styles::{create_element, default_layout_style, inherit, Cascade};
use crate::{Call, Component, Element, Fonts, Input, ValueExtensions};
use log::error;
use serde_json::{Map, Value};
use taffy::prelude::length;
use taffy::style_helpers::TaffyMaxContent;
use taffy::{
    AlignItems, AvailableSpace, Dimension, Display, JustifyContent, NodeId, Size, Style,
    TaffyError, TaffyTree,
};

#[derive(Debug)]
pub enum RenderError {
    Taffy(TaffyError),
}

impl From<TaffyError> for RenderError {
    fn from(error: TaffyError) -> Self {
        RenderError::Taffy(error)
    }
}

impl Component {
    pub fn render(
        &mut self,
        input: &mut Input,
        globals: &mut Map<String, Value>,
    ) -> Result<(NodeId, TaffyTree<Element>), RenderError> {
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
        self.state.active_animators = take(&mut self.state.animators);
        self.render_tree(root, body, globals, input, context, &mut rendering);
        rendering.compute_layout_with_measure(
            root,
            Size::MAX_CONTENT,
            |size, space, _, view, _| measure_text(input, size, space, view),
        )?;
        return Ok((root, rendering));
    }

    fn render_tree(
        &mut self,
        parent_id: NodeId,
        current: Html,
        globals: &mut Map<String, Value>,
        input: &Input,
        context: Sizes,
        layout: &mut TaffyTree<Element>,
    ) {
        if current.text.is_some() {
            self.render_text(parent_id, current, input, globals, layout);
        } else {
            self.render_template(parent_id, current, globals, input, context, layout)
        }
    }

    fn render_text(
        &mut self,
        parent_id: NodeId,
        current: Html,
        input: &Input,
        globals: &mut Map<String, Value>,
        layout: &mut TaffyTree<Element>,
    ) {
        let element_id = ElementId::from(&current);
        let text = current.text.unwrap_or_default();
        let text = interpolate_string(&text, globals, input);
        let style = default_layout_style();
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
        globals: &mut Map<String, Value>,
        input: &Input,
        context: Sizes,
        layout: &mut TaffyTree<Element>,
    ) {
        if let Some(pipe) = current.attrs.get("?") {
            if !is_something(Some(&get_object_value(pipe, globals, input))) {
                return;
            }
        }
        if let Some(ident) = current.attrs.get("!") {
            if is_something(globals.get(ident)) {
                return;
            }
        }
        let repetition = Repetition::from(&globals, current.attrs.get("*"));
        for repeat_value in repetition.values {
            // PUSH STATE
            globals.insert(repetition.key.to_string(), repeat_value.clone());
            // RENDER
            let element_id = ElementId::hash(&current, &repeat_value);
            self.render_element(
                element_id, parent_id, &current, globals, input, context, layout,
            );
            // POP STATE
            globals.remove(&repetition.key);
        }
    }

    fn render_element(
        &mut self,
        element_id: ElementId,
        parent_id: NodeId,
        template: &Html,
        globals: &mut Map<String, Value>,
        input: &Input,
        sizes: Sizes,
        tree: &mut TaffyTree<Element>,
    ) {
        let current_id = tree
            .new_leaf(default_layout_style())
            .expect("id must be created");
        tree.add_child(parent_id, current_id)
            .expect("child must be added");

        let mut current = template.clone();
        for (key, pipe) in &template.attrs {
            if key.starts_with("data-") {
                let string = get_object_value(&pipe, globals, input).as_string();
                current.attrs.insert(key.clone(), string.into());
            }
        }

        let object = Object {
            tag: current.tag.to_string(),
            attrs: current.attrs.clone(),
            text: None,
            pseudo_classes: self.state.load_pseudo_classes(element_id).clone(),
        };
        let mut element = create_element(element_id, object);

        // APPLY STYLES
        let mut layout = default_layout_style();
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
        cascade.apply_styles(current_id, tree, parent, &mut layout, &mut element);

        self.render_output_bindings(&mut element, globals);

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
                self.render_input(current_id, &element, globals, tree);
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
                    self.render_tree(current_id, child, globals, input, context, tree);
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
            ..default_layout_style()
        };
        layout.new_child_of(parent_id, style, element);
    }

    fn render_input(
        &mut self,
        parent_id: NodeId,
        parent: &Element,
        globals: &mut Map<String, Value>,
        layout: &mut TaffyTree<Element>,
    ) {
        let text = match parent.html.attrs.get("value") {
            None => "".to_string(),
            Some(binding) => as_string(globals.get(binding)),
        };
        let element_id = ElementId::child(parent.id, 1);
        let style = default_layout_style();
        let object = Object::text(text);
        let mut element = create_element(element_id, object);
        inherit(&parent, &mut element);
        layout.new_child_of(parent_id, style, element);

        let element_id = ElementId::child(parent.id, 2);
        if parent.html.pseudo_classes.contains("focus") {
            let object = Object::fake();
            let mut element = create_element(element_id, object);
            let mut style = default_layout_style();
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
    fn render_output_bindings(&self, element: &mut Element, globals: &mut Map<String, Value>) {
        let events: &[&str] = match element.html.tag.as_ref() {
            "input" => &["oninput", "onchange"],
            _ => &["onclick"],
        };
        for event in events {
            if let Some(expr) = element.html.attrs.get(*event) {
                let output = eval_call(expr, globals);
                element.listeners.insert(event.to_string(), output);
            }
        }
    }
}

struct ObjectContext<'a> {
    css: &'a str,
    sizes: Sizes,
    variables: HashMap<CssSpan, CssValue>,
}

impl<'a> ObjectContext<'a> {
    pub fn new(css: &'a str, sizes: Sizes) -> Self {
        Self {
            css,
            sizes,
            variables: HashMap::new(),
        }
    }

    pub fn create(&self) -> Self {
        Self {
            css: self.css,
            sizes: self.sizes,
            variables: HashMap::new(),
        }
    }

    pub fn push_variable(&mut self, name: CssSpan, variable: CssValue) {
        self.variables.insert(name, variable);
    }

    pub fn get_variable(&self, name: CssSpan) -> Option<&CssValue> {
        self.variables.get(&name)
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

pub fn is_something(value: Option<&Value>) -> bool {
    match value {
        None => false,
        Some(value) => match value {
            Value::Null => false,
            Value::Bool(value) => *value,
            Value::Number(number) => number.as_f64() != Some(0.0),
            Value::String(value) => !value.is_empty(),
            Value::Array(value) => !value.is_empty(),
            Value::Object(_) => true,
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

pub fn get_object_value(pipe: &str, global: &Map<String, Value>, input: &Input) -> Value {
    let mut value = Value::Null;
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
    let mut scope = global;
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

pub fn interpolate_string(string: &str, value: &Map<String, Value>, input: &Input) -> String {
    let mut result = String::new();
    let mut field = false;
    let mut pipe = String::new();
    for ch in string.chars() {
        if field {
            if ch == '}' {
                result += &get_object_value(&pipe, value, input).as_string();
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

fn eval_call(expression: &str, global_value: &Map<String, Value>) -> Call {
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
                    Err(_) => global_value.get(&value).cloned().unwrap_or(Value::Null),
                };
                arguments.push(value);
                arg = String::new();
            } else {
                arg.push(ch);
            }
        }
    }
    Call {
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

fn measure_text(
    input: &mut Input,
    size: Size<Option<f32>>,
    space: Size<AvailableSpace>,
    element: Option<&mut Element>,
) -> Size<f32> {
    if let Size {
        width: Some(width),
        height: Some(height),
    } = size
    {
        return Size { width, height };
    }
    let element = match element {
        None => return Size::ZERO,
        Some(element) => element,
    };
    if let Some(text) = element.html.text.as_ref() {
        let max_width = size.width.map(Some).unwrap_or_else(|| match space.width {
            AvailableSpace::MinContent => Some(0.0),
            AvailableSpace::MaxContent => None,
            AvailableSpace::Definite(width) => Some(width),
        });
        let [width, height] = match input.fonts.as_mut() {
            None => FakeFonts.measure(&text, &element.text_style, max_width),
            Some(fonts) => fonts.measure(&text, &element.text_style, max_width),
        };
        return Size { width, height };
    }
    Size::ZERO
}
