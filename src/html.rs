use std::os::unix::fs::symlink;
use scraper::node::Element;
use serde_json::{Map, Value};
use taffy::{Dimension, Style};
use crate::models::{Rectangle, TextContext};
use crate::rendering::as_string;

/// Configures the elements or adjust their behavior in various ways to meet HTML experience.
/// 
/// see details: https://developer.mozilla.org/en-US/docs/Web/HTML/Attributes
pub fn adjust(element: &Element, value: &Map<String, Value>, rectangle: &mut Rectangle, style: &mut Style) {
    match element.name.local.trim() {
        "img" => {
            if let Some(width) = element.attr("width") {
                // style.size.width = 
            }
            if let Some(height) = element.attr("height") {
                // style.size.height = parse() ?!
            }
            if let Some(src) = element.attr("src") {
                rectangle.background.image = Some(src.to_string());
            }
        }
        "input" => {
            if let Some(binding) = element.attr("value") {
                let value = as_string(value.get(binding));
                rectangle.text = Some(TextContext::new(&value));
            }
            if style.size.width == Dimension::Auto {
                style.size.width = Dimension::Length(150.0);
            }
        }
        _ => {}
    }
}
