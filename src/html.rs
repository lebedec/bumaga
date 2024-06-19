
use scraper::node::Element;
use taffy::Style;
use crate::models::Rectangle;

/// Configures the elements or adjust their behavior in various ways to meet HTML experience.
/// 
/// see details: https://developer.mozilla.org/en-US/docs/Web/HTML/Attributes
pub fn adjust(element: &Element, rectangle: &mut Rectangle, style: &mut Style) {
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
        _ => {}
    }
}
