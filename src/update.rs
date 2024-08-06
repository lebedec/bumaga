use std::collections::HashSet;
use std::fs;
use std::mem::take;
use std::ops::Add;

use log::error;
use serde_json::Value;
use taffy::prelude::length;
use taffy::style_helpers::TaffyMaxContent;
use taffy::{
    AvailableSpace, Layout, NodeId, Point, Position, PrintTree, Size, Style, TaffyTree,
    TraversePartialTree,
};

use crate::css::read_css_unchecked;
use crate::html::{read_html_unchecked, Html};
use crate::input::FakeFonts;
use crate::models::{ElementId, Object, Sizes};
use crate::rendering::RenderError;
use crate::state::State;
use crate::styles::create_element;
use crate::{Component, Element, Fonts, Input, Keys, Output, Source, LEFT_MOUSE_BUTTON};

impl Component {
    pub fn update(&mut self, mut input: Input) -> Output {
        if let Some(path) = &self.css.path {
            if let Ok(modified) = fs::metadata(path).and_then(|meta| meta.modified()) {
                if modified > self.css.modified {
                    self.css = Source::from_file(read_css_unchecked, path);
                    self.reset_state();
                }
            }
        }
        if let Some(path) = &self.html.path {
            if let Ok(modified) = fs::metadata(path).and_then(|meta| meta.modified()) {
                if modified > self.html.modified {
                    self.html = Source::from_file(read_html_unchecked, path);
                    self.reset_state();
                }
            }
        }

        let mut frame = Output::new();
        let mut value = match input.value.as_object_mut() {
            Some(value) => value.clone(),
            None => {
                error!("input value must be object");
                return frame;
            }
        };

        let (root, tree) = match self.render(&mut input, &mut value) {
            Ok(tree) => tree,
            Err(error) => {
                error!("unable to render component tree, {error:?}");
                return frame;
            }
        };
        self.tree = tree;
        self.root = root;

        fn process(
            tree: &TaffyTree<Element>,
            node: NodeId,
            input: &Input,
            frame: &mut Output,
            state: &mut State,
            mut location: Point<f32>,
        ) {
            let mut layout = *tree.get_final_layout(node);
            let element = match tree.get_node_context(node) {
                None => {
                    error!("unable to traverse node {node:?} has no context");
                    return;
                }
                Some(element) => element,
            };
            let style = match tree.style(node) {
                Ok(style) => style,
                Err(error) => {
                    error!("unable to traverse node {node:?}, {error:?}");
                    return;
                }
            };

            if style.position == Position::Relative {
                layout.location = layout.location.add(location)
            }

            // interaction
            let mut pseudo_classes = HashSet::new();
            if is_element_contains(&layout, input.mouse_position) {
                pseudo_classes.insert("hover".to_string());
                if input.is_mouse_down() {
                    pseudo_classes.insert("active".to_string());
                    state.set_focus(element.id);
                } else if element.html.pseudo_classes.contains("active") {
                    if let Some(call) = element.listeners.get("onclick") {
                        frame.calls.push(call.clone());
                    }
                }
            }
            if state.focus == Some(element.id) {
                pseudo_classes.insert("focus".to_string());

                if element.html.tag == "input" {
                    let value_node = tree
                        .children(node)
                        .expect("input must contain value element")[0];
                    let value_view = tree
                        .get_node_context(value_node)
                        .expect("input value must contain context");
                    let mut value = value_view.html.text.clone().unwrap_or_default();
                    let mut has_changes = false;
                    if !input.characters.is_empty() {
                        has_changes = true;
                        for char in &input.characters {
                            if char != &'\r' {
                                value.push(*char);
                            }
                        }
                    }
                    if input.is_key_pressed(Keys::Backspace) && value.len() > 0 {
                        println!("input ch {:?}", input.characters);
                        has_changes = true;
                        value.pop();
                    }
                    if input.is_key_pressed(Keys::Enter) {
                        if let Some(call) = element.listeners.get("onchange") {
                            let mut call = call.clone();
                            if call.arguments.len() == 0 {
                                call.arguments.push(Value::String(value.clone()));
                            }
                            frame.calls.push(call);
                        }
                    }
                    if has_changes {
                        if let Some(call) = element.listeners.get("oninput") {
                            let mut call = call.clone();
                            if call.arguments.len() == 0 {
                                call.arguments.push(Value::String(value));
                            }
                            frame.calls.push(call);
                        }
                    }
                }
            }
            state.save_pseudo_classes(element.id, pseudo_classes);

            let mut view = element.clone();
            view.layout = layout;
            frame.elements.push(view);
            location = layout.location;
            match tree.children(node) {
                Ok(children) => {
                    for child in children {
                        process(tree, child, input, frame, state, location);
                    }
                }
                Err(error) => {
                    error!("unable to traverse node {node:?}, {error:?}")
                }
            }
        }
        process(
            &self.tree,
            self.root,
            &input,
            &mut frame,
            &mut self.state,
            Point::ZERO,
        );
        frame
    }
}

impl Output {
    fn new() -> Self {
        Self {
            calls: vec![],
            elements: vec![],
        }
    }
}

impl<'f> Input<'f> {
    fn is_mouse_down(&self) -> bool {
        self.mouse_buttons_down.contains(&LEFT_MOUSE_BUTTON)
    }

    fn is_key_down(&self, key: Keys) -> bool {
        self.keys_down
            .iter()
            .position(|key_down| key_down == &key)
            .is_some()
    }

    fn is_key_pressed(&self, key: Keys) -> bool {
        self.keys_pressed
            .iter()
            .position(|key_down| key_down == &key)
            .is_some()
    }
}

fn is_element_contains(layout: &Layout, point: [f32; 2]) -> bool {
    let x = point[0] >= layout.location.x && point[0] <= layout.location.x + layout.size.width;
    let y = point[1] >= layout.location.y && point[1] <= layout.location.y + layout.size.height;
    x && y
}
