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
use crate::input::DummyFonts;
use crate::models::Sizes;
use crate::state::State;
use crate::styles::{create_element, Scrolling};
use crate::{CallOld, Component, Element, Fonts, Input, Keys, Output, Source, ViewError};

impl Component {
    pub fn update(&mut self, mut input: Input) -> Result<Output, ViewError> {
        unimplemented!()
        // self.watch_source_changes();
        // let (root, tree) = self.render_tree(&mut input)?;
        // self.tree = tree;
        // self.root = root;
        // self.tree.compute_layout_with_measure(
        //     self.root,
        //     Size::MAX_CONTENT,
        //     |size, space, _, view, _| measure_text(&mut input, size, space, view),
        // )?;
        // self.state.prune();
        // let mut output = Output::new();
        // self.process_final_layout(self.root, &input, &mut output, Point::ZERO, None);
        // self.handle_user_input(&input, &mut output)?;
        // // TODO: rework output traverse, gathering and element save
        // self.gather(self.root, &mut output);
        // for element in output.elements.iter() {
        //     // println!("ele {:?}", element.html.pseudo_classes);
        //     self.state.save(element);
        // }
        // Ok(output)
    }

    fn watch_source_changes(&mut self) {
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
    }

    fn get_element_mut(&mut self, node: NodeId) -> Result<&mut Element, ViewError> {
        self.tree
            .get_node_context_mut(node)
            .ok_or(ViewError::ElementNotFound)
    }

    fn handle_user_input(&mut self, input: &Input, output: &mut Output) -> Result<(), ViewError> {
        // TODO: propagation
        unimplemented!()
        // if let Some(current) = output.scroll {
        //     let element = self.get_element_mut(current)?;
        //     if let Some(scrolling) = element.scrolling.as_mut() {
        //         scrolling.offset(input.mouse_wheel[0], input.mouse_wheel[1]);
        //     }
        // }
        //
        // // Reset focus
        // let mut focus = None;
        //
        // if let Some(current) = output.hover {
        //     if let Some(previous) = self.state.hover {
        //         if previous != current {
        //             self.state.hover = None;
        //             let element = self.get_element_mut(previous)?;
        //             element.fire("onmouseleave", output);
        //             element.remove("hover");
        //         }
        //     }
        //     if self.state.hover != Some(current) {
        //         self.state.hover = Some(current);
        //         let element = self.get_element_mut(current)?;
        //         element.fire("onmouseenter", output);
        //         element.insert("hover");
        //     }
        //     let element = self.get_element_mut(current)?;
        //     if input.is_mouse_down() {
        //         // Sets focus on the specified element, if it can be focused
        //         match element.tag.as_str() {
        //             "input" => {
        //                 focus = Some(current);
        //             }
        //             _ => {}
        //         }
        //         element.fire("onclick", output);
        //     }
        // }
        //
        // if let Some(previous) = self.state.focus {
        //     let element = self.get_element_mut(previous)?;
        //     let click_out_of_element = input.is_mouse_down()
        //         && !is_element_contains(&element.layout, input.mouse_position);
        //     if focus.is_some() && focus != Some(previous) || click_out_of_element {
        //         element.fire("onblur", output);
        //         element.remove("focus");
        //         self.state.focus = None;
        //     }
        // }
        // if let Some(current) = focus {
        //     if Some(current) != self.state.focus {
        //         let element = self.get_element_mut(current)?;
        //         element.fire("onfocus", output);
        //         element.insert("focus");
        //         self.state.focus = Some(current);
        //     }
        // }
        //
        // if let Some(current) = self.state.focus {
        //     let element = self.get_element_mut(current)?;
        //     match element.tag.as_str() {
        //         "input" => {
        //             let mut value = element.attrs.get("value").cloned().unwrap_or_default();
        //             let mut has_changes = false;
        //             if !input.characters.is_empty() {
        //                 has_changes = true;
        //                 for char in &input.characters {
        //                     if char != &'\r' {
        //                         value.push(*char);
        //                     }
        //                 }
        //             }
        //             if input.is_key_pressed(Keys::Backspace) && value.len() > 0 {
        //                 has_changes = true;
        //                 value.pop();
        //             }
        //             if input.is_key_pressed(Keys::Enter) {
        //                 element.fire_opt("onchange", vec![Value::String(value.clone())], output);
        //             }
        //             if has_changes {
        //                 element.fire_opt("oninput", vec![Value::String(value.clone())], output);
        //             }
        //         }
        //         _ => {}
        //     }
        // }
        //
        // Ok(())
    }

    fn gather(&mut self, node: NodeId, output: &mut Output) {
        unimplemented!()
        // let element = match self.tree.get_node_context_mut(node) {
        //     None => {
        //         error!("unable to traverse node {node:?} has no context");
        //         return;
        //     }
        //     Some(element) => element,
        // };
        // output.elements.push(element.clone());
        // match self.tree.children(node) {
        //     Ok(children) => {
        //         for child in children {
        //             self.gather(child, output);
        //         }
        //     }
        //     Err(error) => {
        //         error!("unable to traverse node {node:?}, {error:?}")
        //     }
        // }
    }

    fn process_final_layout(
        &mut self,
        node: NodeId,
        input: &Input,
        output: &mut Output,
        location: Point<f32>,
        mut clip: Option<Layout>,
    ) {
        unimplemented!()
        // let mut layout = *self.tree.get_final_layout(node);
        // let style = match self.tree.style(node) {
        //     Ok(style) => style,
        //     Err(error) => {
        //         error!("unable to traverse node {node:?}, {error:?}");
        //         return;
        //     }
        // };
        // if style.position == Position::Relative {
        //     layout.location = layout.location.add(location)
        // }
        // let element = match self.tree.get_node_context_mut(node) {
        //     None => {
        //         error!("unable to traverse node {node:?} has no context");
        //         return;
        //     }
        //     Some(element) => element,
        // };
        // element.scrolling = Scrolling::ensure(&layout, &element.scrolling);
        // element.layout = layout;
        // element.clip = clip;
        // if is_element_contains(&element.layout, input.mouse_position) {
        //     output.hover = Some(node);
        //     if element.scrolling.is_some() {
        //         output.scroll = Some(node);
        //     }
        // }
        // let mut location = element.layout.location;
        // if let Some(scrolling) = element.scrolling.as_ref() {
        //     clip = Some(element.layout.clone());
        //     location.x -= scrolling.x;
        //     location.y -= scrolling.y;
        // }
        // match self.tree.children(node) {
        //     Ok(children) => {
        //         for child in children {
        //             self.process_final_layout(child, input, output, location, clip);
        //         }
        //     }
        //     Err(error) => {
        //         error!("unable to traverse node {node:?}, {error:?}")
        //     }
        // }
    }
}

impl Output {
    pub fn new() -> Self {
        Self {
            hover: None,
            scroll: None,
            calls: vec![],
            elements: vec![],
        }
    }
}

impl Element {
    #[inline(always)]
    fn fire(&self, event: &str, output: &mut Output) {
        if let Some(call) = self.listeners_old.get(event).cloned() {
            output.calls.push(call);
        }
    }

    #[inline(always)]
    fn fire_opt(&self, event: &str, arguments: Vec<Value>, output: &mut Output) {
        if let Some(mut call) = self.listeners_old.get(event).cloned() {
            if call.arguments.len() == 0 {
                call.arguments = arguments;
            }
            output.calls.push(call);
        }
    }
}

// impl<'f> Input<'f> {
//     fn is_mouse_down(&self) -> bool {
//         self.mouse_buttons_down.contains(&LEFT_MOUSE_BUTTON)
//     }
//
//     fn is_key_down(&self, key: Keys) -> bool {
//         self.keys_down
//             .iter()
//             .position(|key_down| key_down == &key)
//             .is_some()
//     }
//
//     fn is_key_pressed(&self, key: Keys) -> bool {
//         self.keys_pressed
//             .iter()
//             .position(|key_down| key_down == &key)
//             .is_some()
//     }
// }

fn is_element_contains(layout: &Layout, point: [f32; 2]) -> bool {
    let x = point[0] >= layout.location.x && point[0] <= layout.location.x + layout.size.width;
    let y = point[1] >= layout.location.y && point[1] <= layout.location.y + layout.size.height;
    x && y
}
