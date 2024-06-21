use std::fs;
use std::ops::Add;
use std::path::Path;

use log::error;
use scraper::{Html, Selector};
use serde_json::{Error, Map, Value};
use taffy::{
    AvailableSpace, Layout, NodeId, Point, Position, PrintTree, Size, Style, TaffyResult,
    TaffyTree, TraversePartialTree,
};
use taffy::prelude::length;
use taffy::style_helpers::TaffyMaxContent;

use crate::{Element, Fonts};
use crate::api::{Call, Component, Input, Output};
use crate::input::FakeFonts;
use crate::models::{SizeContext, ViewId};
use crate::rendering::{as_string, render_tree, State};
use crate::styles::{create_view, parse_presentation, pseudo};

impl Component {
    pub fn compile_files<P: AsRef<Path>>(html: P, css: P) -> Self {
        let html_error = format!("unable to read html file {:?}", html.as_ref());
        let html = fs::read_to_string(html).expect(&html_error);
        let css_error = format!("unable to read css file {:?}", css.as_ref());
        let css = fs::read_to_string(css).expect(&css_error);
        Self::compile(&html, &css)
    }

    pub fn compile(html: &str, css: &str) -> Self {
        let presentation = parse_presentation(css);
        let html = Html::parse_document(html);
        let state = State::new();
        let body_selector = Selector::parse("body").expect("body selector must be parsed");
        Self {
            presentation,
            html,
            state,
            body_selector,
        }
    }

    pub fn update(&mut self, mut input: Input) -> Output {
        self.state.element_n = 0;
        let mut frame = Output::new();
        let value = match input.value.as_object_mut() {
            Some(value) => value,
            None => {
                error!("input value must be object");
                return frame;
            }
        };
        let mut rendering = TaffyTree::new();
        let [viewport_width, viewport_height] = input.viewport;
        let viewport_id = ViewId {
            element_n: self.state.element_n,
            hash: 0,
        };
        let viewport_layout = Style {
            size: Size {
                width: length(viewport_width),
                height: length(viewport_height),
            },
            ..Default::default()
        };
        let viewport_view = create_view(viewport_id);
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
        let body = self.html.select(&self.body_selector).next();
        let body = match body {
            Some(body) => body,
            None => {
                error!("unable to update component, body not found");
                return frame;
            }
        };
        render_tree(
            viewport,
            *body,
            value,
            context,
            &self.presentation,
            &mut rendering,
            &mut self.state,
        );
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
            let view = match tree.get_node_context(node) {
                None => {
                    error!("unable to traverse node {node:?} has no context");
                    return;
                }
                Some(view) => view,
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
            let mut pseudo_classes = vec![];
            if is_element_contains(&layout, input.mouse_position) {
                pseudo_classes.push(pseudo(":hover"));
                if input.mouse_button_down {
                    pseudo_classes.push(pseudo(":active"));
                } else if state.has_pseudo_class(view.id, &pseudo(":active")) {
                    if let Some(element) = view.html_element.as_ref() {
                        if let Some(repr) = element.attr("onclick") {
                            frame.calls.push(parse_call(repr, &input.value));
                        }
                    }
                }
            }
            state.set_pseudo_classes(view.id, pseudo_classes);

            let mut view = view.clone();
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

fn is_element_contains(layout: &Layout, point: [f32; 2]) -> bool {
    let x = point[0] >= layout.location.x && point[0] <= layout.location.x + layout.size.width;
    let y = point[1] >= layout.location.y && point[1] <= layout.location.y + layout.size.height;
    x && y
}

fn parse_call(repr: &str, global_value: &Value) -> Call {
    let mut function = String::new();
    let mut arguments = vec![];
    let mut is_function = true;
    let mut arg = String::new();
    for ch in repr.chars() {
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

impl Output {
    fn new() -> Self {
        Self {
            calls: vec![],
            elements: vec![],
        }
    }
}

impl<'f> Input<'f> {
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
        if let Some(text) = element.text.as_ref() {
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
