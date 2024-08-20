use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::time::{Duration, Instant};

use macroquad::input::utils::{register_input_subscriber, repeat_all_miniquad_input};
use macroquad::miniquad::{EventHandler, KeyMods};
use macroquad::prelude::*;
use serde_json::{json, Value};

use bumaga::{
    Borders, Element, ElementFont, Fonts, Fragment, Input, InputEvent, Keys, MouseButtons,
    MyBorder, Rgba, TransformFunction, ValueExtensions, View,
};

#[macroquad::main("macroquad bumaga example")]
async fn main() {
    env_logger::init();
    let subscriber = register_input_subscriber();
    let font = load_ttf_font("../shared/Roboto/Roboto-Regular.ttf")
        .await
        .unwrap();
    let mut fonts = FontSystem { font };
    let mut view = View::watch("../shared/view.html", "../shared/style.css", "../shared/")
        .unwrap()
        .pipe("cap", |value| value.eval_string().to_uppercase().into());
    let mut todos_done = vec![];
    let mut todos = vec![
        "learn bumaga documentation".to_string(),
        "create UI using HTML".to_string(),
        "implement engine".to_string(),
    ];
    // for i in 0..100 {
    //     todos.push(format!("Todo N{i}"));
    // }
    let mut todo = "Enter a todo".to_string();

    loop {
        clear_background(WHITE);
        draw_scene();

        let value = json!({"todos": todos, "todo": todo});
        let mut input = user_input(subscriber)
            .fonts(&mut fonts)
            .time(Duration::from_millis(16))
            .value(value);
        let t1 = Instant::now();
        let output = view.update(input).unwrap();
        // println!("bumaga time: {:?}", t1.elapsed());
        // 34ms initial
        // 20ms without text measure

        draw_element(view.body(), &fonts);

        for call in output {
            match call.signature() {
                ("update", [value]) => todo = value.as_string(),
                ("append", [value]) => todos.push(value.as_string()),
                ("finish", [value]) => todos_done.push(value.clone()),
                ("cancel", [value]) => todos_done.retain(|todo| todo != value),
                ("remove", [value]) => todos.retain(|todo| todo != value),
                (event, arguments) => {
                    println!("CALL {event} {arguments:?}");
                }
            };
        }
        next_frame().await
    }
}

fn draw_element(element: Fragment, fonts: &FontSystem) {
    let [mut x, mut y] = element.element.position;
    let [w, h] = element.element.size;

    for transform in &element.element.transforms {
        match transform {
            TransformFunction::Translate { x: tx, y: ty, .. } => {
                let tx = tx.resolve(w);
                let ty = ty.resolve(h);
                x += tx;
                y += ty;
            }
        }
    }
    if let Some(clip) = element.element.clipping {
        let cx = clip.location.x;
        let cy = clip.location.y;
        let cw = clip.size.width;
        let ch = clip.size.height;
        if !(x >= cx && x + w <= cx + cw && y >= cy && y + h <= cy + ch) {
            return;
        }
    }
    draw_rectangle(x, y, w, h, color(&element.element.background.color));
    draw_borders(&element);
    // draw_line()
    if let Some(text) = element.element.text.as_ref() {
        let text_params = TextParams {
            font_size: element.element.font.size as u16,
            font: Some(&fonts.font),
            color: color(&element.element.color),
            ..Default::default()
        };

        draw_text_ex(
            &text.to_string(),
            x,
            y + fonts.offset_y(&text.to_string(), &element.element.font),
            text_params,
        );
    }

    for fragment in element.children() {
        draw_element(fragment, fonts);
    }
}

fn draw_borders(element: &Fragment) {
    let borders = &element.element.borders;
    let [x, y] = element.element.position;
    let [w, h] = element.element.size;

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
    pub fn offset_y(&self, text: &str, style: &ElementFont) -> f32 {
        let size = measure_text(text, Some(&self.font), style.size as u16, 1.0);
        size.offset_y
    }
}

impl Fonts for FontSystem {
    fn measure(&mut self, text: &str, style: &ElementFont, _max_width: Option<f32>) -> [f32; 2] {
        // NOTE: macroquad does not support width constraint measurement,
        // only single line text will be rendered correctly
        let size = measure_text(text, Some(&self.font), style.size as u16, 1.0);
        [size.width, size.height]
    }
}

pub struct InputAdapter {
    events: Vec<InputEvent>,
}

impl EventHandler for InputAdapter {
    fn update(&mut self) {}

    fn draw(&mut self) {}

    fn mouse_wheel_event(&mut self, x: f32, y: f32) {
        self.events.push(InputEvent::MouseWheel([x, y]))
    }

    fn mouse_motion_event(&mut self, x: f32, y: f32) {
        self.events.push(InputEvent::MouseMove([x, y]))
    }

    fn mouse_button_down_event(&mut self, button: MouseButton, x: f32, y: f32) {
        let button = match button {
            MouseButton::Left => MouseButtons::Left,
            _ => MouseButtons::Right,
        };
        self.events.push(InputEvent::MouseMove([x, y]));
        self.events.push(InputEvent::MouseButtonDown(button))
    }

    fn mouse_button_up_event(&mut self, button: MouseButton, x: f32, y: f32) {
        let button = match button {
            MouseButton::Left => MouseButtons::Left,
            _ => MouseButtons::Right,
        };
        self.events.push(InputEvent::MouseMove([x, y]));
        self.events.push(InputEvent::MouseButtonUp(button))
    }

    fn char_event(&mut self, ch: char, _keymods: KeyMods, _repeat: bool) {
        if !ch.is_control() {
            self.events.push(InputEvent::Char(ch))
        }
    }

    fn key_down_event(&mut self, code: KeyCode, _keymods: KeyMods, _repeat: bool) {
        self.events.push(InputEvent::KeyDown(map_keycode(code)))
    }

    fn key_up_event(&mut self, code: KeyCode, _keymods: KeyMods) {
        self.events.push(InputEvent::KeyUp(map_keycode(code)))
    }
}

pub fn user_input<'f>(subscriber: usize) -> Input<'f> {
    let viewport = [screen_width(), screen_height()];
    let mut adapter = InputAdapter { events: vec![] };
    repeat_all_miniquad_input(&mut adapter, subscriber);
    Input::new().viewport(viewport).events(adapter.events)
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
