use macroquad::prelude::*;
use serde_json::{json, Value};
use std::collections::HashSet;
use std::time::Duration;

use bumaga::{
    Borders, Component, Element, Fonts, Input, Keys, Layout, MyBorder, Rgba, TextStyle,
    TransformFunction, ValueExtensions,
};

#[macroquad::main("macroquad bumaga example")]
async fn main() {
    env_logger::init();
    let font = load_ttf_font("../shared/Roboto/Roboto-Regular.ttf")
        .await
        .unwrap();
    let mut fonts = FontSystem { font };
    let mut component =
        Component::compile_files("../shared/index.html", "../shared/style.css", "../shared/");

    let mut todos_done = vec![];
    let mut todos = vec![
        "learn bumaga documentation".to_string(),
        "create UI using HTML".to_string(),
        "implement engine".to_string(),
    ];
    let mut todo = "Enter a todo".to_string();

    loop {
        clear_background(WHITE);
        draw_scene();
        let value = json!({"todos": todos, "todo": todo});
        let done = todos_done.clone();
        let input = user_input()
            .fonts(&mut fonts)
            .time(Duration::from_millis(16))
            .value(value)
            .pipe("done", move |value| done.contains(&value).into());
        let output = component.update(input).unwrap();
        for element in output.elements {
            draw_element(&element, &fonts);
        }
        for call in output.calls {
            match call.signature() {
                ("update", [value]) => todo = value.as_string(),
                ("append", [value]) => todos.push(value.as_string()),
                ("finish", [value]) => todos_done.push(value.clone()),
                ("cancel", [value]) => todos_done.retain(|todo| todo != value),
                ("remove", [value]) => todos.retain(|todo| todo != value),
                _ => {}
            };
        }
        next_frame().await
    }
}

fn draw_element(element: &Element, fonts: &FontSystem) {
    let mut x = element.layout.location.x;
    let mut y = element.layout.location.y;
    let w = element.layout.size.width;
    let h = element.layout.size.height;
    for transform in &element.transforms {
        match transform {
            TransformFunction::Translate { x: tx, y: ty, .. } => {
                let tx = tx.resolve(element.layout.size.width);
                let ty = ty.resolve(element.layout.size.height);
                x += tx;
                y += ty;
            }
        }
    }
    if let Some(clip) = element.clip {
        let cx = clip.location.x;
        let cy = clip.location.y;
        let cw = clip.size.width;
        let ch = clip.size.height;
        if !(x >= cx && x + w <= cx + cw && y >= cy && y + h <= cy + ch) {
            return;
        }
    }
    draw_rectangle(x, y, w, h, color(&element.background.color));
    draw_borders(&element);
    // draw_line()
    if let Some(text) = element.html.text.as_ref() {
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

fn draw_borders(element: &Element) {
    let borders = &element.borders;
    let layout = &element.layout;
    let x = layout.location.x;
    let y = layout.location.y;
    let w = layout.size.width;
    let h = layout.size.height;
    if let Some(border) = borders.top() {
        draw_line(x, y, x + w, y, border.width, color(&border.color))
    }
    if let Some(border) = borders.bottom() {
        draw_line(x, y + h, x + w, y + h, border.width, color(&border.color))
    }
    if let Some(border) = borders.left() {
        draw_line(x, y, x, y + h, border.width, color(&border.color))
    }
    if let Some(border) = borders.right() {
        draw_line(x + w, y, x + w, y + h, border.width, color(&border.color))
    }
}

fn color(rgba: &Rgba) -> Color {
    Color::from_rgba(rgba[0], rgba[1], rgba[2], rgba[3])
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

pub fn user_input<'f>() -> Input<'f> {
    let viewport = [screen_width(), screen_height()];
    let mut characters = vec![];
    while let Some(character) = get_char_pressed() {
        if character == ' ' || !character.is_whitespace() && character != '\u{7f}' {
            characters.push(character);
        }
    }
    let keys_down = get_keys_down().into_iter().map(map_keycode).collect();
    let keys_up = get_keys_released().into_iter().map(map_keycode).collect();
    let keys_pressed = get_keys_pressed().into_iter().map(map_keycode).collect();
    let mouse_position = mouse_position().into();
    let wheel = mouse_wheel();

    let mut buttons_down = vec![];
    let mut buttons_up = vec![];
    let buttons = [
        (MouseButton::Left, 0),
        (MouseButton::Right, 1),
        (MouseButton::Middle, 2),
    ];
    for (button, code) in buttons {
        if is_mouse_button_down(button) {
            buttons_down.push(code);
        }
        if is_mouse_button_released(button) {
            buttons_up.push(code);
        }
    }
    Input::new()
        .viewport(viewport)
        .mouse_position(mouse_position)
        .mouse_buttons_down(buttons_down)
        .mouse_buttons_up(buttons_up)
        .mouse_wheel([wheel.0, wheel.1])
        .characters(characters)
        .keys_down(keys_down)
        .keys_up(keys_up)
        .keys_pressed(keys_pressed)
}

pub fn map_keycode(code: KeyCode) -> Keys {
    match code {
        // UI keys
        KeyCode::Escape => Keys::Escape,
        // Editing keys
        KeyCode::Backspace => Keys::Backspace,
        KeyCode::Delete => Keys::Delete,
        KeyCode::Insert => Keys::Insert,
        // Whitespace keys
        KeyCode::Enter => Keys::Enter,
        KeyCode::Tab => Keys::Tab,
        // Navigation keys
        KeyCode::Up => Keys::ArrowUp,
        KeyCode::Down => Keys::ArrowDown,
        KeyCode::Left => Keys::ArrowLeft,
        KeyCode::Right => Keys::ArrowRight,
        KeyCode::End => Keys::End,
        KeyCode::Home => Keys::Home,
        KeyCode::PageDown => Keys::PageDown,
        KeyCode::PageUp => Keys::PageUp,
        // Modifier keys
        KeyCode::LeftAlt => Keys::Alt,
        KeyCode::RightAlt => Keys::Alt,
        KeyCode::CapsLock => Keys::CapsLock,
        KeyCode::LeftControl => Keys::Ctrl,
        KeyCode::RightControl => Keys::Ctrl,
        KeyCode::LeftShift => Keys::Shift,
        KeyCode::RightShift => Keys::Shift,
        _ => Keys::Unknown,
    }
}

fn draw_scene() {
    set_camera(&Camera3D {
        position: vec3(-10., 10., 0.),
        up: vec3(0., 1., 0.),
        target: vec3(0., 0., 0.),
        ..Default::default()
    });
    draw_grid(20, 1., BLACK, GRAY);
    draw_cube_wires(vec3(0., 1., -6.), vec3(2., 2., 2.), DARKGREEN);
    draw_cube_wires(vec3(0., 1., 6.), vec3(2., 2., 2.), DARKBLUE);
    draw_cube_wires(vec3(2., 1., 2.), vec3(2., 2., 2.), YELLOW);
    draw_cube(vec3(2., 0., -2.), vec3(0.4, 0.4, 0.4), None, BLACK);
    draw_sphere(vec3(-8., 0., 0.), 1., None, BLUE);
    set_default_camera();
}
