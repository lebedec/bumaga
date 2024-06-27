use serde_json::{Map, Value};
use taffy::Style;

use crate::Element;

pub fn apply_html_attributes(
    element: &scraper::node::Element,
    value: &Map<String, Value>,
    view: &mut Element,
    style: &mut Style,
) {
    match element.name.local.trim() {
        "img" => {
            if let Some(width) = element.attr("width") {
                // style.size.width =
            }
            if let Some(height) = element.attr("height") {
                // style.size.height = parse() ?!
            }
        }
        "input" => {
            // if let Some(binding) = element.attr("value") {
            //     let value = as_string(value.get(binding));
            //     view.text = Some(value);
            // }
            // if style.size.width == Dimension::Auto {
            //     style.size.width = Dimension::Length(150.0);
            // }
        }
        _ => {}
    }
}
