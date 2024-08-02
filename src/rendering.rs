use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::mem;
use std::process::id;

use lightningcss::values::color::{CssColor, RGBA};
use log::error;
use serde_json::{Map, Value};
use taffy::{AlignItems, Dimension, Display, JustifyContent, NodeId, Size, Style, TaffyTree};

use crate::animation::apply_animation_rules;
use crate::html::{match_rule, Object};
use crate::models::{ElementId, Presentation, SizeContext};
use crate::state::State;
use crate::styles::{
    apply_layout_rules, apply_view_rules, create_view, default_layout_style, inherit, pseudo,
};
use crate::{Call, Component, Element, Input, ValueExtensions};

impl Component {
    pub fn render_tree<'a, 'b>(
        &'a mut self,
        parent_id: NodeId,
        current: Object,
        globals: &mut Map<String, Value>,
        input: &Input,
        context: SizeContext,
        layout: &mut TaffyTree<Element>,
    ) {
        if let Some(text) = current.text.clone() {
            if !text.is_empty() {
                // fake text element
                self.state.element_n += 1;
                let element_id = ElementId {
                    element_n: self.state.element_n,
                    hash: 0,
                };
                let text = interpolate_string(text, globals, input);
                let style = default_layout_style();
                let parent = layout.get_node_context(parent_id).expect("context must be");
                let mut view = create_view(element_id, &current);
                inherit(&parent, &mut view);
                layout.new_child_of(parent_id, style, view.clone());
            }
        } else {
            self.state.element_n += 1;
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
            let no_array = vec![Value::Null];
            let no_array_key = String::new();
            let repeat = if let Some(ident) = current.attrs.get("*") {
                match as_array(globals.get(ident)) {
                    None => return,
                    Some(array) => (ident.to_string(), array.clone()), // TODO: remove clone
                }
            } else {
                (no_array_key, no_array)
            };
            let (repeat_key, repeat_values) = repeat;

            for repeat_value in repeat_values {
                let element_id = ElementId {
                    element_n: self.state.element_n,
                    // array of object (especially big objects) can reduce performance
                    // TODO: use key attribute like React
                    hash: hash_value(&repeat_value),
                };

                let current_id = layout
                    .new_leaf(default_layout_style())
                    .expect("id must be created");
                layout
                    .add_child(parent_id, current_id)
                    .expect("child must be added");

                // PUSH STATE
                if !repeat_key.is_empty() {
                    // TODO: replace value ?
                    // TODO: remove clone ?
                    globals.insert(repeat_key.clone(), repeat_value.clone());
                }

                let original_element = current.clone();
                let mut element_mut = current.clone();
                let pseudo_classes = self.state.get_pseudo_classes(element_id);
                if !pseudo_classes.is_empty() {
                    let defined = match element_mut.attrs.get("class") {
                        None => String::new(),
                        Some(classes) => classes.trim().to_string(),
                    };
                    let mut pseudo_classes = pseudo_classes.clone();
                    pseudo_classes.insert(0, defined);
                    let result = pseudo_classes.join(" ");
                    element_mut.attrs.insert("class".to_string(), result.into());
                }
                for (key, pipe) in &current.attrs {
                    if key.starts_with("data-") {
                        let string = get_object_value(&pipe, globals, input).as_string();
                        element_mut.attrs.insert(key.clone(), string.into());
                    }
                }
                let element = current.clone();

                let mut style = default_layout_style();
                let mut view = create_view(element_id, &original_element);
                layout
                    .set_node_context(current_id, Some(view.clone()))
                    .expect("context must be set");

                let parent = layout.get_node_context(parent_id).expect("context must be");

                for ruleset in &self.presentation.content.rules {
                    if match_rule(&ruleset.style, current_id, layout) {
                        let props = &ruleset.style.declarations.declarations;
                        apply_layout_rules(props, &mut style, context);
                        apply_view_rules(props, &parent, &mut view, context, &self.resources);
                        apply_animation_rules(
                            props,
                            &mut view,
                            &mut self.state.active_animators,
                            &mut self.state.animators,
                            &self.presentation.content.animations,
                        );
                    }
                }
                // apply animation
                let mut no_animators = vec![];
                let animators = self
                    .state
                    .animators
                    .get_mut(&element_id)
                    .unwrap_or(&mut no_animators);
                for animator in animators {
                    let props = animator.update(input.time.as_secs_f32());
                    // println!(
                    //     "APPLY {} t{} p{}",
                    //     animator.id(),
                    //     animator.time,
                    //     props.len()
                    // );
                    apply_layout_rules(&props, &mut style, context);
                    apply_view_rules(&props, &parent, &mut view, context, &self.resources);
                }

                // parse output binding
                // NOTE: must be in rendering cycle because scope contains repeated values
                // TODO: analyze performance issues (skip call render if no events)
                // Configures the elements or adjust their behavior in various ways to meet HTML experience.
                //
                // see details: https://developer.mozilla.org/en-US/docs/Web/HTML/Attributes
                match view.html.tag.as_ref() {
                    "input" => {
                        if let Some(expr) = element.attrs.get("oninput") {
                            view.listeners
                                .insert("oninput".to_string(), render_call(expr, globals));
                        }
                        if let Some(expr) = element.attrs.get("onchange") {
                            view.listeners
                                .insert("onchange".to_string(), render_call(expr, globals));
                        }
                        // if let Some(binding) = element.attr("value") {
                        //     let value = as_string(value.get(binding));
                        //     view.text = Some(value);
                        // }
                        style.display = Display::Flex;
                        style.align_items = Some(AlignItems::Center);
                    }
                    _ => {
                        if let Some(expr) = element.attrs.get("onclick") {
                            view.listeners
                                .insert("onclick".to_string(), render_call(expr, globals));
                        }
                    }
                }
                //

                layout
                    .set_node_context(current_id, Some(view.clone()))
                    .expect("update node context");

                // special rendering
                match view.html.tag.as_str() {
                    "img" => {
                        self.state.element_n += 1;
                        let object_element_id = ElementId {
                            element_n: self.state.element_n,
                            hash: 0,
                        };
                        let empty = "undefined.png".to_string();
                        let src = element.attrs.get("src").unwrap_or(&empty);
                        let src = format!("{}{}", self.resources, src);
                        let mut object_style = default_layout_style();
                        let fake_object = Object {
                            tag: "".to_string(),
                            attrs: Default::default(),
                            text: None,
                            children: vec![],
                        };
                        let mut object_element = create_view(object_element_id, &fake_object);
                        object_element.background.image = Some(src);
                        object_style.size = Size {
                            width: Dimension::Percent(1.0),
                            height: Dimension::Percent(1.0),
                        };
                        // object element size width, height
                        // object element position
                        layout.new_child_of(current_id, object_style, object_element);
                    }
                    "input" => {
                        // inner text elements
                        self.state.element_n += 1;
                        let text = match element.attrs.get("value") {
                            None => "".to_string(),
                            Some(binding) => as_string(globals.get(binding)),
                        };
                        let value_element_id = ElementId {
                            element_n: self.state.element_n,
                            hash: 0,
                        };
                        let value_style = default_layout_style();
                        let fake_object = Object {
                            tag: "".to_string(),
                            attrs: Default::default(),
                            text: Some(text),
                            children: vec![],
                        };
                        let mut value_view = create_view(value_element_id, &fake_object);
                        inherit(&view, &mut value_view);
                        layout.new_child_of(current_id, value_style, value_view.clone());

                        self.state.element_n += 1;
                        let caret_element_id = ElementId {
                            element_n: self.state.element_n,
                            hash: 0,
                        };
                        if self.state.has_pseudo_class(view.id, &pseudo(":focus")) {
                            let mut caret_style = default_layout_style();
                            let fake_object = Object {
                                tag: "".to_string(),
                                attrs: Default::default(),
                                text: None,
                                children: vec![],
                            };
                            let mut caret_view = create_view(caret_element_id, &fake_object);
                            caret_style.size.width = Dimension::Length(1.0);
                            caret_style.size.height =
                                Dimension::Length(value_view.text_style.font_size);
                            caret_view.background.color = value_view.color;
                            layout.new_child_of(current_id, caret_style, caret_view.clone());
                        }
                    }
                    _ => {
                        for child in current.children.clone() {
                            let mut context = context;
                            context.parent_font_size = view.text_style.font_size;
                            self.render_tree(current_id, child, globals, input, context, layout);
                        }
                    }
                }

                // POP STATE
                if !repeat_key.is_empty() {
                    globals.remove(&repeat_key);
                }
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

pub fn hash_value(value: &Value) -> u64 {
    let mut hasher = DefaultHasher::new();
    // available since v1.0.118
    value.hash(&mut hasher);
    hasher.finish()
    // match value {
    //     Value::Null => 0,
    //     Value::Bool(value) => {
    //         if *value {
    //             1
    //         } else {
    //             0
    //         }
    //     }
    //     Value::Number(value) => {
    //         let value = value.as_f64().unwrap_or(0.0);
    //         integer_decode(value).hash(&mut hasher);
    //         hasher.finish()
    //     }
    //     Value::String(value) => {
    //         value.hash(&mut hasher);
    //         hasher.finish()
    //     }
    //     Value::Array(array) => {
    //         let hashes: Vec<u64> = array.iter().map(hash_value).collect();
    //         hashes.hash(&mut hasher);
    //         hasher.finish()
    //     }
    //     Value::Object(object) => {
    //         let mut hashes: Vec<(String, u64)> = vec![];
    //         for (key, value) in object {
    //             hashes.push((key.to_string(), hash_value(value)));
    //         }
    //         hashes.hash(&mut hasher);
    //         hasher.finish()
    //     }
    // }
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

pub fn interpolate_string(string: String, value: &Map<String, Value>, input: &Input) -> String {
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

fn render_call(expression: &str, global_value: &Map<String, Value>) -> Call {
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
