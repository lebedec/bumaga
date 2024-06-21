use macroquad::miniquad::window::screen_size;
use macroquad::prelude::*;
use serde_json::json;

use bumaga::{Borders, Component, CssColor, Element, Fonts, Input, Layout, MyBorder, TextStyle};

#[macroquad::main("macroquad bumaga example")]
async fn main() {
    env_logger::init();
    let font = load_ttf_font("../shared/Roboto/Roboto-Regular.ttf")
        .await
        .unwrap();
    let mut fonts = FontSystem { font };
    let mut component = Component::compile_files("../shared/index.html", "../shared/style.css");
    let mut todos = vec![
        "learn bumaga".to_string(),
        "create UI using HTML".to_string(),
        "implement engine".to_string(),
    ];
    let mut todo = "Enter a todo".to_string();
    loop {
        clear_background(WHITE);
        let value = json!({"todos": todos, "todo": todo});
        let input = Input::new()
            .fonts(&mut fonts)
            .value(value)
            .viewport(screen_size().into());
        let output = component.update(input);
        for element in output.elements {
            draw_rectangle(
                element.layout.location.x,
                element.layout.location.y,
                element.layout.size.width,
                element.layout.size.height,
                color(&element.background.color),
            );
            draw_borders(&element);
            // draw_line()
            if let Some(text) = element.text {
                let text_params = TextParams {
                    font_size: element.text_style.font_size as u16,
                    font: Some(&fonts.font),
                    ..Default::default()
                };
                draw_text_ex(
                    &text,
                    element.layout.location.x,
                    element.layout.location.y + fonts.offset_y(&text, &element.text_style),
                    text_params,
                );
            }
        }
        next_frame().await
    }
}

fn draw_borders(element: &Element) {
    let borders = &element.borders;
    let layout = &element.layout;
    let x = layout.location.x;
    let y = layout.location.y;
    let w = layout.size.width;
    let h = layout.size.height;
    if let Some(border) = borders.top.as_ref() {
        draw_line(x, y, x + w, y, border.width, color(&border.color))
    }
    if let Some(border) = borders.bottom.as_ref() {
        draw_line(x, y + h, x + w, y + h, border.width, color(&border.color))
    }
    if let Some(border) = borders.left.as_ref() {
        draw_line(x, y, x, y + h, border.width, color(&border.color))
    }
    if let Some(border) = borders.left.as_ref() {
        draw_line(x + w, y, x + w, y + h, border.width, color(&border.color))
    }
}

fn color(css: &CssColor) -> Color {
    match css {
        CssColor::RGBA(color) => Color::from_rgba(color.red, color.green, color.blue, color.alpha),
        _ => RED,
    }
}

struct FontSystem {
    font: Font,
}

impl FontSystem {
    pub fn offset_y(&self, text: &str, style: &TextStyle) -> f32 {
        let size = measure_text(text, Some(&self.font), style.font_size as u16, 1.0);
        size.offset_y
    }
}

impl Fonts for FontSystem {
    fn measure(&mut self, text: &str, style: &TextStyle, _max_width: Option<f32>) -> [f32; 2] {
        // NOTE: macroquad does not support width constraint measurement,
        // only single line text will be rendered correctly
        let size = measure_text(text, Some(&self.font), style.font_size as u16, 1.0);
        [size.width, size.height]
    }
}
