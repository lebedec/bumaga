mod html;
mod styles;
mod rendering;
mod models;
mod api;

use crate::html::adjust;
use crate::styles::{apply_rectangle_rules, apply_style_rules, default_layout_style, default_rectangle_style, inherit, parse_presentation};
use ego_tree::NodeRef;
use lightningcss::printer::PrinterOptions;
use lightningcss::properties::background::{
    Background, BackgroundAttachment, BackgroundClip, BackgroundOrigin, BackgroundPosition,
    BackgroundRepeat, BackgroundSize,
};
use lightningcss::rules::style::StyleRule;
use lightningcss::rules::CssRule;
use lightningcss::stylesheet::{ParserOptions, StyleSheet};
use lightningcss::traits::{Op, ToCss};
use lightningcss::values::color::{CssColor, RGBA};
use lightningcss::values::image::Image;
use log::error;
use scraper::{ElementRef, Html, Node, Selector};
use serde_json::{json, Map, Value};
use std::fs;
use taffy::prelude::{length, TaffyMaxContent};
use taffy::{
    AvailableSpace, Dimension, Display, FlexDirection, FlexWrap, GridAutoFlow, GridPlacement, Line,
    NodeId, Overflow, Point, Position, PrintTree, Rect, Size, Style, TaffyResult, TaffyTree,
};
use crate::models::{Presentation, Rectangle, SizeContext};
use crate::rendering::{render_tree, State};

pub fn do_something() {
    let mut value: Value = json!({
        "name": "Alice",
        "nested": {
            "propertyA": 42,
            "propertyB": 43
        },
        "items": ["a", 32, "b", 33],
        "visible": true,
        "collection": [
            {"value": "v1", "name": "value 1"},
            {"value": "v2", "name": "value 2"},
        ]
    });

    let template = fs::read_to_string("./assets/index.html").expect("index.html");
    let presentation = fs::read_to_string("./assets/style.css").expect("style.css");
    let presentation = parse_presentation(&presentation);

    let html = Html::parse_document(&template);
    let body_selector = Selector::parse("body").expect("body selector");
    let body = html.select(&body_selector).next().expect("body element");
    
    
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
            default_rectangle_style(),
        )
        .unwrap();
    let context = SizeContext {
        level: 0,
        root_font_size: 16.0,
        parent_font_size: 16.0,
        viewport_width: 800.0,
        viewport_height: 100.0,
    };
    let mut state = State::new();
    render_tree(viewport, *body, value.as_object_mut().expect("must be object"), context, &presentation, &mut rendering, &mut state);
    println!("rendering nodes: {}", rendering.total_node_count());
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

    let mut result = String::new();
    result += "<style>body { font-family: \"Courier New\"; font-size: 14px; }</style>\n";

    fn print_node(tree: &TaffyTree<Rectangle>, node: NodeId, output: &mut String) {
        let layout = tree.get_final_layout(node);
        let rectangle = match tree.get_node_context(node) {
            None => {
                error!("unable to traverse node {node:?} has no context");
                return;
            }
            Some(rectangle) => rectangle,
        };
        let k = &rectangle.key;
        let x = layout.location.x;
        let y = layout.location.y;
        let w = layout.size.width;
        let h = layout.size.height;
        let empty = String::new();
        let t = rectangle.text.as_ref().unwrap_or(&empty);
        println!(
            "{k} bg {:?} cs{:?} sc{:?} s{:?}",
            rectangle.background.color, layout.content_size, layout.scrollbar_size, layout.size
        );
        let mut bg = rectangle
            .background
            .color
            .to_css_string(PrinterOptions::default())
            .expect("css color");
        if let Some(img) = rectangle.background.image.as_ref() {
            println!("img {img}");
            bg = format!("url({img})");
        }
        let element = format!("<div key=\"{k}\" style=\"position: fixed; top: {y}px; left: {x}px; width: {w}px; height: {h}px; background: {bg};\">{t}</div>\n");
        *output += &element;
        match tree.children(node) {
            Ok(children) => {
                for child in children {
                    print_node(tree, child, output);
                }
            }
            Err(error) => {
                error!("unable to traverse node {node:?}, {error:?}")
            }
        }
    }

    print_node(&rendering, viewport, &mut result);

    fs::write("./assets/result.html", result).expect("result written");

    println!("ok");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        do_something()
    }
}
