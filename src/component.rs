use log::error;
use scraper::{Html, Selector};
use serde_json::{Error, Map, Value};
use taffy::{AvailableSpace, Layout, NodeId, PrintTree, Size, Style, TaffyTree};
use taffy::prelude::length;
use taffy::style_helpers::TaffyMaxContent;
use crate::api::{Call, Component, Element, Frame, Input};
use crate::models::{ElementId, Rectangle, SizeContext};
use crate::rendering::{as_string, render_tree, State};
use crate::styles::{create_rectangle, parse_presentation, pseudo};

impl Component {
    pub fn compile(html: &str, css: &str) -> Self {
        let presentation = parse_presentation(css);
        let html = Html::parse_document(html);
        let state = State::new();
        Self {
            presentation,
            html,
            state
        }
    }

    pub fn update(&mut self, mut interop: Input) -> Frame {
        let body_selector = Selector::parse("body").expect("body selector");
        let body = self.html.select(&body_selector).next().expect("body element");

        // reset each update
        self.state.element_n = 0;

        let root_element_id = ElementId {
            element_n: self.state.element_n,
            hash: 0,
        };
        let mut rendering = TaffyTree::new();
        let viewport = rendering
            .new_leaf_with_context(
                Style {
                    size: Size {
                        width: length(800.0),
                        height: length(100.0),
                    },
                    ..Default::default()
                },
                create_rectangle(root_element_id),
            )
            .unwrap();
        let context = SizeContext {
            level: 0,
            root_font_size: 16.0,
            parent_font_size: 16.0,
            viewport_width: 800.0,
            viewport_height: 100.0,
        };

        let value = interop.value.as_object_mut().expect("must be object");
        render_tree(
            viewport,
            *body,
            value,
            context,
            &self.presentation,
            &mut rendering,
            &mut self.state
        );
        
        let measure_fn = |size: Size<Option<f32>>, available_space: Size<AvailableSpace>, _node_id: NodeId, rectangle: Option<&mut Rectangle>, _style: &Style| {
            if let Size {
                width: Some(width),
                height: Some(height),
            } = size
            {
                return Size { width, height };
            }
            match rectangle {
                None => {}
                Some(rectangle) => {
                    if let Some(text) = rectangle.text.as_ref() {

                        let width_constraint = size.width.map(Some).unwrap_or_else(|| match available_space.width {
                            AvailableSpace::MinContent => Some(0.0),
                            AvailableSpace::MaxContent => None,
                            AvailableSpace::Definite(width) => Some(width),
                        });
                        
                        let m = interop.fonts.measure(&text.text, &rectangle.text_style, width_constraint);
                        return Size {width: m[0], height: m[1]};
                    }
                }
            }
            Size::ZERO
        };
        rendering
            .compute_layout_with_measure(viewport, Size::MAX_CONTENT, measure_fn)
            .unwrap();
        let mut frame = Frame {
            calls: vec![],
            elements: vec![],
        };
        fn traverse(tree: &TaffyTree<Rectangle>, node: NodeId, input: &Input, frame: &mut Frame, state: &mut State) {
            let layout = tree.get_final_layout(node);
            let rectangle = match tree.get_node_context(node) {
                None => {
                    error!("unable to traverse node {node:?} has no context");
                    return;
                }
                Some(rectangle) => rectangle,
            };

            // interaction
            let mut pseudo_classes = vec![];
            if is_element_contains(layout, input.mouse_position) {
                pseudo_classes.push(pseudo(":hover"));
                if input.mouse_button_down {
                    pseudo_classes.push(pseudo(":active"));
                } else if state.has_pseudo_class(rectangle.id, &pseudo(":active")) {
                    if let Some(element) = rectangle.element.as_ref() {
                        if let Some(repr) = element.attr("onclick") {
                            frame.calls.push(parse_call(repr, &input.value));
                        }
                    }
                }
            }
            state.set_pseudo_classes(rectangle.id, pseudo_classes);

            frame.elements.push(Element {
                rectangle: rectangle.clone(),
                layout: *layout
            });
            match tree.children(node) {
                Ok(children) => {
                    for child in children {
                        traverse(tree, child, input, frame, state);
                    }
                }
                Err(error) => {
                    error!("unable to traverse node {node:?}, {error:?}")
                }
            }
        }
        traverse(&rendering, viewport, &interop, &mut frame, &mut self.state);
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
                    Err(_) => {
                        global_value.get(&value).cloned().unwrap_or(Value::Null)
                    }
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