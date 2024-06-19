use std::fs;
use std::time::Duration;
use lightningcss::printer::PrinterOptions;
use lightningcss::traits::ToCss;
use log::error;
use scraper::{Html, Selector};
use serde_json::Value;
use taffy::{AvailableSpace, Layout, NodeId, PrintTree, Size, Style, TaffyTree};
use taffy::prelude::length;
use taffy::style_helpers::TaffyMaxContent;
use crate::models::{ElementId, Presentation, Rectangle, SizeContext};
use crate::rendering::{render_tree, State};
use crate::styles::{create_rectangle, parse_presentation};

pub struct Component {
    presentation: Presentation,
    html: Html,
    state: State,
}

impl Component {
    pub fn new(template: String, presentation: String) -> Self {
        let presentation = parse_presentation(&presentation);
        let html = Html::parse_document(&template);
        let state = State::new();
        Self {
            presentation,
            html,
            state
        }
    }
    
    pub fn update(&mut self, mut input: Input) -> Frame {
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
        
        let value = input.value.as_object_mut().expect("must be object");
        render_tree(
            viewport, 
            *body, 
            value, 
            context, 
            &self.presentation, 
            &mut rendering, 
            &mut self.state
        );
        struct FontSystem {
            letter_h: f32,
            letter_w: f32,
        }
        // monospaced, font-size: 14px
        let font_system = FontSystem {
            letter_h: 15.5,
            letter_w: 8.43,
        };
        rendering
            .compute_layout_with_measure(
                viewport,
                Size::MAX_CONTENT,
                |size, available_space, _node_id, rectangle, _style| {
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
                                let width_constraint =
                                    size.width.unwrap_or_else(|| match available_space.width {
                                        AvailableSpace::MinContent => 0.0,
                                        AvailableSpace::MaxContent => f32::INFINITY,
                                        AvailableSpace::Definite(width) => width,
                                    });
                                let max_letters =
                                    (width_constraint / font_system.letter_w).floor() as usize;
                                if max_letters > 0 {
                                    let lines = text.len() / max_letters + 1;
                                    let width = (text.len() as f32 * font_system.letter_w)
                                        .min(width_constraint);
                                    let height = lines as f32 * font_system.letter_h;

                                    return taffy::Size { width, height };
                                }

                                println!("size {size:?} [{text}] available space {available_space:?}")
                            }
                        }
                    }
                    Size::ZERO
                },
            )
            .unwrap();
        let mut frame = Frame {
            calls: vec![],
            elements: vec![],
        };
        fn traverse(tree: &TaffyTree<Rectangle>, node: NodeId, frame: &mut Frame) {
            let layout = tree.get_final_layout(node);
            let rectangle = match tree.get_node_context(node) {
                None => {
                    error!("unable to traverse node {node:?} has no context");
                    return;
                }
                Some(rectangle) => rectangle,
            };
            frame.elements.push(Element {
                rectangle: rectangle.clone(),
                layout: *layout
            });
            match tree.children(node) {
                Ok(children) => {
                    for child in children {
                        traverse(tree, child, frame);
                    }
                }
                Err(error) => {
                    error!("unable to traverse node {node:?}, {error:?}")
                }
            }
        }
        traverse(&rendering, viewport, &mut frame);
        frame
    }
}

pub struct Input {
    value: Value,
    time: Duration,
    keys: Vec<String>,
    mouse_position: [f32; 2]
}

impl Input {
    pub fn new() -> Input {
        Input {
            value: Value::Null,
            time: Duration::from_micros(0),
            keys: vec![],
            mouse_position: [0.0, 0.0]
        }
    }
    
    pub fn value(mut self, value: Value) -> Input {
        self.value = value;
        self
    }

    pub fn time(mut self, time: Duration) -> Input {
        self.time = time;
        self
    }

    pub fn mouse(mut self, mouse_position: [f32; 2]) -> Input {
        self.mouse_position = mouse_position;
        self
    }
}

/// It is a mechanism that allows a Bumaga component to request
/// interaction event handling in application.
pub struct Call {
    /// The identifier of event handler (function name probably).
    function: String,
    /// The JSON-like argument from component template.
    arguments: Vec<Value>
}

pub struct Element {
    pub layout: Layout,
    pub rectangle: Rectangle
}

pub struct Frame {
    pub calls: Vec<Call>,
    pub elements: Vec<Element>
}

