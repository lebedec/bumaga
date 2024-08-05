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
use crate::html::{read_html_unchecked, Dom};
use crate::input::FakeFonts;
use crate::models::{ElementId, Object, SizeContext};
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
        let mut rendering = TaffyTree::new();
        let [viewport_width, viewport_height] = input.viewport;
        let viewport_id = ElementId::fake();
        let viewport_layout = Style {
            size: Size {
                width: length(viewport_width),
                height: length(viewport_height),
            },
            ..Default::default()
        };
        let fake_object = Object::fake();
        let viewport_view = create_element(viewport_id, fake_object);
        let context = SizeContext {
            root_font_size: viewport_view.text_style.font_size,
            parent_font_size: viewport_view.text_style.font_size,
            viewport_width,
            viewport_height,
        };
        let viewport = rendering.new_leaf_with_context(viewport_layout, viewport_view);
        let viewport = match viewport {
            Ok(viewport) => viewport,
            Err(error) => {
                error!("unable to create viewport, {error:?}");
                return frame;
            }
        };
        let html = self.html.content.clone();
        let body = html.children.last().cloned().expect("body must be found");
        self.state.active_animators = take(&mut self.state.animators);
        self.render_tree(viewport, body, &mut value, &input, context, &mut rendering);
        let result = rendering.compute_layout_with_measure(
            viewport,
            Size::MAX_CONTENT,
            |size, space, _, view, _| input.measure_text(size, space, view),
        );
        if let Err(error) = result {
            error!("unable to layout component, {error:?}");
            return frame;
        };

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
        let body = rendering.child_ids(viewport).next().expect("must be");
        process(
            &rendering,
            body,
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

    fn measure_text(
        &mut self,
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
            let [width, height] = match self.fonts.as_mut() {
                None => FakeFonts.measure(&text, &element.text_style, max_width),
                Some(fonts) => fonts.measure(&text, &element.text_style, max_width),
            };
            return Size { width, height };
        }
        Size::ZERO
    }
}

fn is_element_contains(layout: &Layout, point: [f32; 2]) -> bool {
    let x = point[0] >= layout.location.x && point[0] <= layout.location.x + layout.size.width;
    let y = point[1] >= layout.location.y && point[1] <= layout.location.y + layout.size.height;
    x && y
}
