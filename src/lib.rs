mod styles;

use std::fs;
use ego_tree::iter::Edge;
use ego_tree::NodeRef;
use lightningcss::properties::background::{Background, BackgroundOrigin};
use lightningcss::properties::Property;
use lightningcss::rules::CssRule;
use lightningcss::rules::style::StyleRule;
use lightningcss::stylesheet::{ParserOptions, StyleSheet};
use lightningcss::traits::ToCss;
use log::error;
use scraper::{ElementRef, Html, Node, Selector, StrTendril};
use scraper::selectable::Selectable;
use serde_json::{json, Value};
use taffy::{Display, FlexDirection, NodeId, Size, Style, TaffyResult, TaffyTree};
use taffy::prelude::{length, TaffyMaxContent};
use crate::styles::{apply_rectangle_rules, apply_style_rules};


pub fn make_style() -> Style {
    Style {
        flex_direction: FlexDirection::Column,
        size: Size { width: length(800.0), height: length(600.0) },
        ..Default::default()
    }
}


#[derive(Clone, Copy)]
pub struct SizeContext {

}

#[derive(Clone)]
pub struct Rectangle<'i> {
    background: Background<'i>
}

impl Default for Rectangle<'_> {
    fn default() -> Self {
        Self {
            background: Background {
                image: Default::default(),
                color: Default::default(),
                position: Default::default(),
                repeat: Default::default(),
                size: Default::default(),
                attachment: Default::default(),
                origin: BackgroundOrigin::PaddingBox,
                clip: Default::default(),
            }
        }
    }
}

pub fn render_tree<'p>(parent_id: NodeId, current: NodeRef<Node>, level: usize, presentation: &'p Presentation, layout: &mut TaffyTree<Rectangle<'p>>) {
    match current.value() {
        Node::Text(text) => {
            println!("{parent_id:?} t {}", text.text)

        }
        Node::Element(element) => {
            println!("{parent_id:?} {} {}", "-".repeat(level), element.name.local);
            let mut style = Style::default();
            let mut rectangle = Rectangle::default();
            for rule in &presentation.rules {
                if rule.selector.matches(&ElementRef::wrap(current).expect("node is element")) {
                    let context = SizeContext {};
                    apply_style_rules(rule, &mut style, context);
                    apply_rectangle_rules(rule, &mut rectangle);
                }
            }

            let current_id = match layout.new_leaf_with_context(style, rectangle) {
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
                render_tree(current_id, child, level + 1, presentation, layout);
            }
        }
        _ => {}
    }
}

pub struct Ruleset<'i> {
    pub selector: Selector,
    pub style: StyleRule<'i>
}

pub struct Presentation<'i> {
    pub rules: Vec<Ruleset<'i>>
}

impl Presentation<'_> {

    pub fn parse(code: &str) -> Presentation {
        let sheet = StyleSheet::parse(code, ParserOptions::default()).unwrap();
        let mut rules = vec![];
        for rule in sheet.rules.0 {
            match rule {
                CssRule::Style(style) => {
                    let css_selector = style.selectors.to_string();
                    let selector = Selector::parse(&css_selector).expect("selector must be: ");
                    let style = Ruleset {
                        selector,
                        style,
                    };
                    rules.push(style);
                }
                _ => {}
            }
        }
        Presentation {
            rules
        }
    }
}

pub fn do_something() {
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

    let template = fs::read_to_string("./src/index.html").expect("index.html");
    let presentation = fs::read_to_string("./src/style.css").expect("style.css");
    let presentation = Presentation::parse(&presentation);


    let html = Html::parse_document(&template);
    let body_selector = Selector::parse("body").expect("body selector");
    let body = html.select(&body_selector).next().expect("body element");
    let mut rendering = TaffyTree::new();
    let document = rendering.new_leaf(
        Style {
            size: Size { width: length(800.0), height: length(100.0) },
            ..Default::default()
        },
    ).unwrap();
    render_tree(document, *body, 0, &presentation, &mut rendering);
    println!("rendering nodes: {}", rendering.total_node_count());

    rendering.compute_layout_with_measure(
        document,
        Size::MAX_CONTENT,
        |_size, _available_space, _node_id, _node_context, _style| {
            if let Size { width: Some(width), height: Some(height) } = _size {
                return Size { width, height };
            }
            Size::ZERO
        }
    ).unwrap();


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
