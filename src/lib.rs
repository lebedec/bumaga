mod html;
mod styles;

use crate::html::adjust;
use crate::styles::{apply_rectangle_rules, apply_style_rules, inherit};
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

#[derive(Clone, Copy)]
pub struct SizeContext {
    level: usize,
    root_font_size: f32,
    parent_font_size: f32,
    viewport_width: f32,
    viewport_height: f32,
}

#[derive(Clone)]
pub struct MyBackground {
    /// The background image.
    pub image: Option<String>,
    /// The background color.
    pub color: CssColor,
    /// The background position.
    pub position: BackgroundPosition,
    /// How the background image should repeat.
    pub repeat: BackgroundRepeat,
    /// The size of the background image.
    pub size: BackgroundSize,
    /// The background attachment.
    pub attachment: BackgroundAttachment,
    /// The background origin.
    pub origin: BackgroundOrigin,
    /// How the background should be clipped.
    pub clip: BackgroundClip,
}

#[derive(Clone)]
pub struct Rectangle {
    key: String,
    background: MyBackground,
    color: RGBA,
    font_size: f32,
    text: Option<String>,
}

impl Default for Rectangle {
    fn default() -> Self {
        Self {
            key: "".to_string(),
            background: MyBackground {
                image: None,
                color: Default::default(),
                position: Default::default(),
                repeat: Default::default(),
                size: Default::default(),
                attachment: Default::default(),
                origin: BackgroundOrigin::PaddingBox,
                clip: Default::default(),
            },
            color: RGBA::new(255, 255, 255, 1.0),
            font_size: 16.0,
            text: None,
        }
    }
}

pub fn default_layout_style() -> Style {
    Style {
        display: Display::Block,
        overflow: Point {
            x: Overflow::Visible,
            y: Overflow::Visible,
        },
        scrollbar_width: 0.0,
        position: Position::Relative,
        inset: Rect::auto(),
        margin: Rect::zero(),
        padding: Rect::zero(),
        border: Rect::zero(),
        size: Size::auto(),
        min_size: Size::auto(),
        max_size: Size::auto(),
        aspect_ratio: None,
        gap: Size::zero(),
        align_items: None,
        align_self: None,
        justify_items: None,
        justify_self: None,
        align_content: None,
        justify_content: None,
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::NoWrap,
        flex_grow: 0.0,
        flex_shrink: 1.0,
        flex_basis: Dimension::Auto,
        ..Default::default()
    }
}

pub fn is_something(value: Option<&Value>) -> bool {
    match value {
        None => false,
        Some(value) => match value {
            Value::Null => false,
            Value::Bool(value) => *value,
            Value::Number(number) => number.as_f64() != Some(0.0),
            Value::String(value) => !value.is_empty(),
            Value::Array(value) => !value.is_empty(),
            Value::Object(_) => true,
        },
    }
}

pub fn get_property<'v>(value: &'v Value, name: &str) -> Option<&'v Value> {
    value.get(name)
}

pub fn as_array(value: Option<&Value>) -> Option<&Vec<Value>> {
    match value {
        None => None,
        Some(value) => value.as_array()
    }
}

pub fn as_string(value: Option<&Value>) -> String {
    match value {
        None => String::new(),
        Some(value) => match value {
            Value::Null => String::new(),
            Value::Bool(value) => value.to_string(),
            Value::Number(value) => value.to_string(),
            Value::String(value) => value.clone(),
            Value::Array(_) => String::from("[array]"),
            Value::Object(_) => String::from("[object]"),
        }
    }
}

pub fn render_text(text: String, value: &Map<String, Value>) -> String {
    let mut result = String::new();
    let mut field = false;
    let mut field_name = String::new();
    for ch in text.chars() {
        if field {
            if ch == '}' {
                result += &as_string(value.get(&field_name));
                field = false;
            } else {
                field_name.push(ch);
            }
        } else {
            if ch == '{' {
                field = true;
                field_name = String::new();
            }
            if !field {
                result.push(ch);
            }
        }
    }
    result
}

struct RepeatContext {
    
}

pub fn render_tree<'p>(
    parent_id: NodeId,
    current: NodeRef<Node>,
    value: &mut Map<String, Value>,
    context: SizeContext,
    presentation: &'p Presentation,
    layout: &mut TaffyTree<Rectangle>,
) {
    match current.value() {
        Node::Text(text) => {
            let text = text.text.trim().to_string();
            let text = render_text(text, value);
            if !text.is_empty() {
                // fake text element
                println!("{parent_id:?} t {}", text);
                let style = default_layout_style();
                let parent_rectangle = layout.get_node_context(parent_id).expect("context must be");
                let mut rectangle = Rectangle::default();
                rectangle.key = "text".to_string();
                rectangle.text = Some(text);
                inherit(&parent_rectangle, &mut rectangle);

                let current_id = match layout.new_leaf_with_context(style, rectangle.clone()) {
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
            }
        }
        Node::Element(element) => {
            // println!("{parent_id:?} {} {}", "-".repeat(context.level), element.name.local);

            if let Some(ident) = element.attr("?") {
                if !is_something(value.get(ident)) {
                    return;
                }
            }
            if let Some(ident) = element.attr("!") {
                if is_something(value.get(ident)) {
                    return;
                }
            }
            let no_array = vec![Value::Null];
            let no_array_key = String::new();
            let repeat = if let Some(ident) = element.attr("*") {
                match as_array(value.get(ident)) {
                    None => return,
                    Some(array) => (ident.to_string(), array.clone()) // TODO: remove clone
                }
            } else {
                (no_array_key, no_array)
            };
            let (repeat_key, repeat_values) = repeat; 
            
            for repeat_value in repeat_values {
                if !repeat_key.is_empty() {
                    // TODO: replace value ?
                    // TODO: remove clone ?
                    value.insert(repeat_key.clone(), repeat_value.clone()); 
                }

                let mut style = default_layout_style();
                let mut rectangle = Rectangle::default();
                let parent_rectangle = layout.get_node_context(parent_id).expect("context must be");
                rectangle.key = element.name.local.to_string();
                for rule in &presentation.rules {
                    if rule
                        .selector
                        .matches(&ElementRef::wrap(current).expect("node is element"))
                    {
                        apply_style_rules(rule, &mut style, context);
                        apply_rectangle_rules(rule, &parent_rectangle, &mut rectangle, context);
                    }
                }
                adjust(element, &mut rectangle, &mut style);

                let current_id = match layout.new_leaf_with_context(style, rectangle.clone()) {
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
                    if let Some(text) = child.value().as_text() {
                        if !child.has_siblings() {
                            let inner_text = render_text(text.text.to_string(), value);
                            layout.get_node_context_mut(current_id).unwrap().text =
                                Some(inner_text);
                            break;
                        }
                    }
                    let mut context = context;
                    context.parent_font_size = rectangle.font_size;
                    context.level += 1;
                    render_tree(current_id, child, value, context, presentation, layout);
                }
                
                if !repeat_key.is_empty() {
                    value.remove(&repeat_key);
                }
            }
        }
        _ => {}
    }
}

pub struct Ruleset<'i> {
    pub selector: Selector,
    pub style: StyleRule<'i>,
}

pub struct Presentation<'i> {
    pub rules: Vec<Ruleset<'i>>,
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
                    let style = Ruleset { selector, style };
                    rules.push(style);
                }
                _ => {}
            }
        }
        Presentation { rules }
    }
}

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
    let presentation = Presentation::parse(&presentation);

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
            Rectangle::default(),
        )
        .unwrap();
    let context = SizeContext {
        level: 0,
        root_font_size: 16.0,
        parent_font_size: 16.0,
        viewport_width: 800.0,
        viewport_height: 100.0,
    };
    render_tree(viewport, *body, value.as_object_mut().expect("must be object"), context, &presentation, &mut rendering);
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
