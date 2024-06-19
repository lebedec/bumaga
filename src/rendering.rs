use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::mem;
use std::ops::Deref;
use std::ptr::hash;
use ego_tree::{NodeMut, NodeRef};
use log::error;
use scraper::{ElementRef, Node, StrTendril};
use scraper::node::Element;
use serde_json::{Map, Value};
use taffy::{NodeId, TaffyTree};
use crate::html::adjust;
use crate::models::{ElementId, Presentation, Rectangle, SizeContext};
use crate::styles::{apply_rectangle_rules, apply_style_rules, default_layout_style, create_rectangle, inherit, pseudo};
use html5ever::{Attribute, LocalName, QualName, ns};
use html5ever::namespace_url;


pub struct State {
    pub element_n: usize,
    pub pseudo_classes: HashMap<ElementId, Vec<String>>
}

static NO_PSEUDO_CLASSES: Vec<String> = vec![];

impl State {
    
    pub fn new() -> Self {
        State {
            element_n: 0,
            pseudo_classes: HashMap::new(),
        }
    }
    
    pub fn get_pseudo_classes(&self, element_id: ElementId) -> &Vec<String> {
        self.pseudo_classes.get(&element_id).unwrap_or(&NO_PSEUDO_CLASSES)
    }
    
    pub fn set_pseudo_classes(&mut self, element_id: ElementId, classes: Vec<String>) {
        self.pseudo_classes.insert(element_id, classes);
    }
}

pub fn render_tree<'p>(
    parent_id: NodeId,
    current: NodeRef<Node>,
    value: &mut Map<String, Value>,
    context: SizeContext,
    presentation: &'p Presentation,
    layout: &mut TaffyTree<Rectangle>,
    state: &mut State
) {

    match current.value() {
        Node::Text(text) => {
            let text = text.text.trim().to_string();
            if !text.is_empty() {
                // fake text element
                state.element_n += 1;
                let element_id = ElementId {
                    element_n: state.element_n,
                    hash: 0,
                };
                let text = interpolate_string(text, value);
                println!("{parent_id:?} t {}", text);
                let style = default_layout_style();
                let parent_rectangle = layout.get_node_context(parent_id).expect("context must be");
                let mut rectangle = create_rectangle(element_id);
                rectangle.key = "text".to_string();
                rectangle.text = Some(text);
                inherit(&parent_rectangle, &mut rectangle);

                let current_id = match layout.new_leaf_with_context(style, rectangle.clone()) {
                    Ok(node_id) => node_id,
                    Err(error) => {
                        error!("unable to create rendering node, {}", error);
                        return;
                    }
                };
                if let Err(error) = layout.add_child(parent_id, current_id) {
                    error!("unable to append rendering node, {}", error);
                    return;
                }
            }
        }
        Node::Element(element) => {
            state.element_n += 1;
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
                    Some(array) => (ident.to_string(), array.clone()) // TODO: remove clone
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
                    element_n: state.element_n,
                    // array of object (especially big objects) can reduce performance
                    // TODO: use key attribute like React
                    hash: hash_value(&repeat_value),
                };

                
                // PUSH STATE
                let original_element = element.clone();
                let mut element_mut = element.clone();
                let class_attr = qual("class");
                match element_mut.attrs.get(&class_attr) {
                    None => {}
                    Some(classes) => {
                        let mut result = String::new();
                        result += classes.trim();
                        result += " ";
                        result += &pseudo(":hover");
                        element_mut.attrs.insert(class_attr, result.into());
                    }
                }
                *current_mut = Node::Element(element_mut);
                let element = current_mut.as_element().unwrap();
                if !repeat_key.is_empty() {
                    // TODO: replace value ?
                    // TODO: remove clone ?
                    value.insert(repeat_key.clone(), repeat_value.clone());
                }

                let mut style = default_layout_style();
                let mut rectangle = create_rectangle(element_id);
                let parent_rectangle = layout.get_node_context(parent_id).expect("context must be");
                rectangle.key = element.name.local.to_string();


                for rule in &presentation.rules {
                    if rule
                        .selector
                        .matches(&ElementRef::wrap(current).expect("node is element"))
                    {
                        apply_style_rules(rule, &mut style, context);
                        apply_rectangle_rules(rule, &parent_rectangle, &mut rectangle, context);
                    }
                }
                adjust(element, &mut rectangle, &mut style);

                // if is_hover {
                //     state.set_state(element_id, pseudo(":hover"))
                // }

                let current_id = match layout.new_leaf_with_context(style, rectangle.clone()) {
                    Ok(node_id) => node_id,
                    Err(error) => {
                        error!("unable to create rendering node, {}", error);
                        return;
                    }
                };
                if let Err(error) = layout.add_child(parent_id, current_id) {
                    error!("unable to append rendering node, {}", error);
                    return;
                }

                for child in current.children() {
                    if let Some(text) = child.value().as_text() {
                        if !child.has_siblings() {
                            let inner_text = interpolate_string(text.text.to_string(), value);
                            layout.get_node_context_mut(current_id).unwrap().text =
                                Some(inner_text);
                            break;
                        }
                    }
                    let mut context = context;
                    context.parent_font_size = rectangle.font_size;
                    context.level += 1;
                    render_tree(current_id, child, value, context, presentation, layout, state);
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
        Some(value) => value.as_array()
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
        }
    }
}

pub fn hash_value(value: &Value) -> u64 {
    let mut hasher = DefaultHasher::new();
    match value {
        Value::Null => 0,
        Value::Bool(value) => if *value { 1 } else { 0 },
        Value::Number(value) => {
            let value = value.as_f64().unwrap_or(0.0);
            integer_decode(value).hash(&mut hasher);
            hasher.finish()
        },
        Value::String(value) => {
            value.hash(&mut hasher);
            hasher.finish()
        },
        Value::Array(array) => {
            let hashes: Vec<u64> = array.iter().map(hash_value).collect();
            hashes.hash(&mut hasher);
            hasher.finish()
        },
        Value::Object(object) => {
            let mut hashes: Vec<(String, u64)> = vec![];
            for (key, value) in object {
                hashes.push((key.to_string(), hash_value(value)));
            }
            hashes.hash(&mut hasher);
            hasher.finish()
        }
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
    QualName::new(
        None,
        ns!(),
        LocalName::from(name)
    )
}
