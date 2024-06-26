use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::mem;

use ego_tree::NodeRef;
use html5ever::{LocalName, ns, QualName};
use html5ever::namespace_url;
use lightningcss::values::color::{CssColor, RGBA};
use log::error;
use scraper::{ElementRef, Node};
use serde_json::{Map, Value};
use taffy::{Dimension, NodeId, Size, Style, TaffyTree};

use crate::{Call, Component, Element};
use crate::html::apply_html_attributes;
use crate::models::{ElementId, Presentation, SizeContext};
use crate::state::State;
use crate::styles::{
    apply_layout_rules, apply_view_rules, create_view, default_layout_style, inherit, pseudo,
};

impl Component {
    pub fn render_tree<'a, 'b>(
        &'a mut self,
        parent_id: NodeId,
        current: NodeRef<'b, Node>,
        value: &mut Map<String, Value>,
        context: SizeContext,
        layout: &mut TaffyTree<Element>,
    ) {
        match current.value() {
            Node::Text(text) => {
                let text = text.text.trim().to_string();
                if !text.is_empty() {
                    // fake text element
                    self.state.element_n += 1;
                    let element_id = ElementId {
                        element_n: self.state.element_n,
                        hash: 0,
                    };
                    let text = interpolate_string(text, value);
                    let style = default_layout_style();
                    let parent = layout.get_node_context(parent_id).expect("context must be");
                    let mut view = create_view(element_id);
                    view.text = Some(text);
                    view.tag = "".to_string();
                    inherit(&parent, &mut view);

                    layout.new_child_of(parent_id, style, view.clone());
                }
            }
            Node::Element(element) => {
                self.state.element_n += 1;
                if let Some(ident) = element.attr("?") {
                    if !is_something(value.get(ident)) {
                        return;
                    }
                }
                if let Some(ident) = element.attr("!") {
                    if is_something(value.get(ident)) {
                        return;
                    }
                }
                let no_array = vec![Value::Null];
                let no_array_key = String::new();
                let repeat = if let Some(ident) = element.attr("*") {
                    match as_array(value.get(ident)) {
                        None => return,
                        Some(array) => (ident.to_string(), array.clone()), // TODO: remove clone
                    }
                } else {
                    (no_array_key, no_array)
                };
                let (repeat_key, repeat_values) = repeat;

                // pointer need to in-place modification of DOM element attributes via instantiation
                // NOTE: safe implementation requires rework scrapy HTML parsing and this render
                #[allow(invalid_reference_casting)]
                let current_mut = unsafe {
                    let ptr = current.value() as *const Node as *mut Node;
                    &mut *ptr
                };

                for repeat_value in repeat_values {
                    let element_id = ElementId {
                        element_n: self.state.element_n,
                        // array of object (especially big objects) can reduce performance
                        // TODO: use key attribute like React
                        hash: hash_value(&repeat_value),
                    };

                    // PUSH STATE
                    let original_element = element.clone();
                    let mut element_mut = element.clone();
                    let class_attr = qual("class");
                    let pseudo_classes = self.state.get_pseudo_classes(element_id);
                    if !pseudo_classes.is_empty() {
                        let defined = match element_mut.attrs.get(&class_attr) {
                            None => String::new(),
                            Some(classes) => classes.trim().to_string(),
                        };
                        let mut pseudo_classes = pseudo_classes.clone();
                        pseudo_classes.insert(0, defined);
                        let result = pseudo_classes.join(" ");
                        element_mut.attrs.insert(class_attr, result.into());
                    }
                    *current_mut = Node::Element(element_mut);
                    let element = current_mut.as_element().unwrap();
                    if !repeat_key.is_empty() {
                        // TODO: replace value ?
                        // TODO: remove clone ?
                        value.insert(repeat_key.clone(), repeat_value.clone());
                    }

                    let mut style = default_layout_style();
                    let mut view = create_view(element_id);
                    view.html_element = Some(original_element.clone());
                    view.tag = element.name.local.to_string();
                    let parent = layout.get_node_context(parent_id).expect("context must be");
                    let matching_element = &ElementRef::wrap(current).expect("node is element");
                    for rule in &self.presentation.rules {
                        if rule.selector.matches(matching_element) {
                            apply_layout_rules(rule, &mut style, context);
                            apply_view_rules(rule, &parent, &mut view, context, &self.resources);
                        }
                    }
                    apply_html_attributes(element, value, &mut view, &mut style);

                    // parse output binding
                    // NOTE: must be in rendering cycle because scope contains repeated values
                    // TODO: analyze performance issues (skip call render if no events)
                    match view.tag.as_ref() {
                        "input" => {
                            if let Some(expr) = element.attr("oninput") {
                                view.listeners
                                    .insert("oninput".to_string(), render_call(expr, value));
                            }
                            if let Some(expr) = element.attr("onchange") {
                                view.listeners
                                    .insert("onchange".to_string(), render_call(expr, value));
                            }
                        }
                        _ => {
                            if let Some(expr) = element.attr("onclick") {
                                view.listeners
                                    .insert("onclick".to_string(), render_call(expr, value));
                            }
                        }
                    }
                    //

                    let current_id = match layout.new_child_of(parent_id, style, view.clone()) {
                        None => return,
                        Some(current_id) => current_id,
                    };

                    // special rendering

                    match view.tag.as_str() {
                        "img" => {
                            self.state.element_n += 1;
                            let object_element_id = ElementId {
                                element_n: self.state.element_n,
                                hash: 0,
                            };
                            let empty = "undefined.png";
                            let src = element.attr("src").unwrap_or(empty);
                            let src = format!("{}{}", self.resources, src);
                            let mut object_style = default_layout_style();
                            let mut object_element = create_view(object_element_id);
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
                            // self.state.element_n += 1;
                            // let text = match element.attr("value") {
                            //     None => "".to_string(),
                            //     Some(binding) => as_string(value.get(binding)),
                            // };
                            // let element_id = ElementId {
                            //     element_n: self.state.element_n,
                            //     hash: 0,
                            // };
                            // let text = interpolate_string(text, value);
                            // let style = default_layout_style();
                            // let parent =
                            //     layout.get_node_context(parent_id).expect("context must be");
                            // let mut view = create_view(element_id);
                            // view.text = Some(text);
                            // view.tag = "".to_string();
                            // inherit(&parent, &mut view);
                            // layout.new_child_of(parent_id, style, view.clone());
                        }
                        _ => {
                            for child in current.children() {
                                let mut context = context;
                                context.parent_font_size = view.text_style.font_size;
                                self.render_tree(current_id, child, value, context, layout);
                            }
                        }
                    }

                    // POP STATE
                    *current_mut = Node::Element(original_element.clone());
                    if !repeat_key.is_empty() {
                        value.remove(&repeat_key);
                    }
                }
            }
            _ => {}
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

pub fn interpolate_string(string: String, value: &Map<String, Value>) -> String {
    let mut result = String::new();
    let mut field = false;
    let mut field_name = String::new();
    for ch in string.chars() {
        if field {
            if ch == '}' {
                result += &as_string(value.get(&field_name));
                field = false;
            } else {
                field_name.push(ch);
            }
        } else {
            if ch == '{' {
                field = true;
                field_name = String::new();
            }
            if !field {
                result.push(ch);
            }
        }
    }
    result
}

pub fn qual(name: &str) -> QualName {
    QualName::new(None, ns!(), LocalName::from(name))
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
