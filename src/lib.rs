mod html;
mod styles;
mod rendering;
mod models;
mod api;

use crate::html::adjust;
use crate::styles::{apply_rectangle_rules, apply_style_rules, default_layout_style, create_rectangle, inherit, parse_presentation};
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
use crate::api::{Component, Input};
use crate::models::{Presentation, Rectangle, SizeContext};
use crate::rendering::{render_tree, State};

pub fn do_something() {
    // awake
    let template = fs::read_to_string("./assets/index.html").expect("index.html");
    let css = fs::read_to_string("./assets/style.css").expect("style.css");
    let mut component = Component::new(template, css);
    
    // update cycle
    let value: Value = json!({
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
    let input = Input::new().value(value).mouse([15.0, 15.0]);
    let frame = component.update(input);

    // drawing
    let mut result = String::new();
    result += "<style>body { font-family: \"Courier New\"; font-size: 14px; }</style>\n";
    for element in frame.elements {
        let rectangle = &element.rectangle;
        let layout = &element.layout;
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
        let record = format!("<div key=\"{k}\" style=\"position: fixed; top: {y}px; left: {x}px; width: {w}px; height: {h}px; background: {bg};\">{t}</div>\n");
        result += &record;
    }
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
