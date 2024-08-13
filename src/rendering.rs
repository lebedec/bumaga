use std::collections::HashMap;
use std::hash::Hash;

use serde_json::{json, Value};
use taffy::{NodeId, TaffyTree};

use crate::html::{ElementBinding, Html, TextBinding, TextSpan};
use crate::styles::{create_element, default_layout};
use crate::view_model::{Binding, Bindings, Schema};
use crate::{BindingParams, Call, Component, Element, Handler, Input, TextContent, ViewError};

impl Component {
    pub fn render_node(
        template: Html,
        tree: &mut TaffyTree<Element>,
        bindings: &mut Bindings,
        locals: &mut HashMap<String, String>,
        schema: &mut Schema,
    ) -> Result<NodeId, ViewError> {
        if let Some(text) = template.text {
            Self::render_text(text, tree, bindings, locals, schema)
        } else {
            Self::render_template(template, tree, bindings, locals, schema)
        }
    }

    pub fn render_text(
        text: TextBinding,
        tree: &mut TaffyTree<Element>,
        bindings: &mut Bindings,
        locals: &mut HashMap<String, String>,
        schema: &mut Schema,
    ) -> Result<NodeId, ViewError> {
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
                    let params = BindingParams::Text(node, index);
                    let binding = Binding {
                        params,
                        pipe: binder.pipe.clone(),
                    };
                    bindings.entry(path).or_default().push(binding);
                    binder.to_string()
                }
            })
            .collect();
        let mut element = create_element(node);
        element.text = Some(TextContent { spans });
        tree.set_node_context(node, Some(element))?;
        Ok(node)
    }

    pub fn render_template(
        template: Html,
        tree: &mut TaffyTree<Element>,
        bindings: &mut Bindings,
        locals: &mut HashMap<String, String>,
        schema: &mut Schema,
    ) -> Result<NodeId, ViewError> {
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
    ) -> Result<NodeId, ViewError> {
        let layout = default_layout();
        let node = tree.new_leaf(layout)?;
        let mut element = create_element(node);
        element.tag = template.tag.clone();
        for binding in template.bindings {
            match binding {
                ElementBinding::None(key, value) => {
                    element.attrs.insert(key, value);
                }
                ElementBinding::Attribute(key, binder) => {
                    let path = schema.field(&binder, locals);
                    let params = BindingParams::Attribute(node, key.clone());
                    let binding = Binding {
                        params,
                        pipe: binder.pipe.clone(),
                    };
                    bindings.entry(path).or_default().push(binding);
                    element.attrs.insert(key, binder.to_string());
                }
                ElementBinding::Callback(event, function, argument) => {
                    let argument = schema.field(&argument, locals);
                    let handler = Handler { function, argument };
                    element.listeners.insert(event.clone(), handler);
                }
                ElementBinding::Visibility(visible, binder) => {
                    let path = schema.field(&binder, locals);
                    let params = BindingParams::Visibility(node, visible);
                    let binding = Binding {
                        params,
                        pipe: binder.pipe.clone(),
                    };
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
                let params = BindingParams::Repeat(node, start, count);
                let binding = Binding {
                    params,
                    pipe: binder.pipe.clone(),
                };

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
        element.children = children.clone();
        tree.set_node_context(node, Some(element))?;
        tree.set_children(node, &children)?;
        Ok(node)
    }

    fn render_img(&mut self, parent_id: NodeId, parent: &Element, layout: &mut TaffyTree<Element>) {
        // let element_id = ElementId::fake();
        // let empty = "undefined.png".to_string();
        // let src = parent.attrs.get("src").unwrap_or(&empty);
        // let src = format!("{}{}", self.resources, src);
        // let mut element = create_element(element_id);
        // element.background.image = Some(src);
        // let style = Style {
        //     size: Size {
        //         width: Dimension::Percent(1.0),
        //         height: Dimension::Percent(1.0),
        //     },
        //     ..default_layout()
        // };
        // layout.new_child_of(parent_id, style, element);
    }

    fn render_input(
        &mut self,
        text: String,
        parent_id: NodeId,
        parent: &Element,
        layout: &mut TaffyTree<Element>,
    ) {
        // let element_id = ElementId::child(parent.id, 1);
        // let style = default_layout();
        // let mut element = create_element(element_id);
        // element.text = Some(TextContent { spans: vec![text] });
        // inherit(&parent, &mut element);
        // layout.new_child_of(parent_id, style, element);
        //
        // let element_id = ElementId::child(parent.id, 2);
        // if parent.pseudo_classes.contains("focus") {
        //     let mut element = create_element(element_id);
        //     let mut style = default_layout();
        //     style.size.width = Dimension::Length(1.0);
        //     style.size.height = Dimension::Length(element.text_style.font_size);
        //     element.background.color = element.color;
        //     layout.new_child_of(parent_id, style, element);
        // }
    }
}

//
// pub fn get_object_value(pipe: &str, input: &Input) -> serde_json::Value {
//     let mut value = serde_json::Value::Null;
//     let segments: Vec<&str> = pipe.split("|").map(&str::trim).collect();
//     let getters = match segments.get(0) {
//         None => {
//             error!("empty pipe");
//             return value;
//         }
//         Some(path) => {
//             let getters: Vec<String> = path
//                 .split(".")
//                 .map(|getter| getter.trim().to_string())
//                 .collect();
//             if getters.len() == 0 {
//                 error!("empty getters");
//                 return value;
//             }
//             getters
//         }
//     };
//     let mut scope = &input.value;
//     for i in 0..getters.len() - 1 {
//         let getter = &getters[i];
//         scope = match scope.get(getter).and_then(|v| v.as_object()) {
//             None => {
//                 error!("nested attribute '{getter}' not object");
//                 return value;
//             }
//             Some(nested) => nested,
//         }
//     }
//     let attr = &getters[getters.len() - 1];
//     value = match scope.get(attr) {
//         None => {
//             error!("attribute '{attr}' not found");
//             return value;
//         }
//         Some(value) => value.clone(),
//     };
//     for name in segments.iter().skip(1) {
//         match input.transformers.get(*name) {
//             None => error!("transformer {name} not registered"),
//             Some(transform) => value = transform(value),
//         }
//     }
//     value
// }

fn eval_call(expression: &str, input: &Input) -> Call {
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
    Call {
        function,
        arguments,
    }
}
