use std::{fs, process};
use std::collections::{HashMap, HashSet};
use std::mem::take;

use core_graphics_types::geometry::CGSize;
use objc::rc::autoreleasepool;
use serde_json::json;
use skia_safe::{
    Canvas, Color4f, Data, Font, FontMgr, Image, Paint, Picture, Point, Rect, Size, Typeface,
};
use skia_safe::utils::text_utils::Align;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};
use winit::event::{DeviceEvent, ElementState, KeyEvent, MouseButton};
use winit::keyboard::{KeyCode, PhysicalKey};

use bumaga::{Component, Element, Fonts, Input, Keys, Rgba, TextStyle};

use crate::metal::create_metal_layer;

mod metal;

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;
const WINDOW_SCALE: f32 = 2.0;

fn serve() {}

fn main() {
    let events_loop = EventLoop::new().expect("failed to create event loop");

    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
        .with_title("Skia+Metal+Winit bumaga example".to_string())
        .build(&events_loop)
        .expect("failed to create window");

    let mut metal = create_metal_layer(&window);

    let mut fonts = FontSystem::new();
    let mut images = ImageSystem::new();

    let mut component =
        Component::compile_files("../shared/index.html", "../shared/style.css", "../shared/");
    let mut todos = vec![
        "learn bumaga documentation".to_string(),
        "create UI using HTML".to_string(),
        "implement engine".to_string(),
    ];
    let mut todo = "Enter a todo".to_string();
    let mut events = vec![];
    let mut mouse_position = [0.0, 0.0];

    let mut event_handler = |event| match event {
        Event::WindowEvent { event, .. } => match event {
            // WindowEvent::CloseRequested => window_target.exit(),
            WindowEvent::Resized(size) => {
                metal
                    .layer
                    .set_drawable_size(CGSize::new(size.width as f64, size.height as f64));
                window.request_redraw()
            }
            WindowEvent::RedrawRequested => {
                metal.redraw(|canvas| {
                    canvas.scale((WINDOW_SCALE, WINDOW_SCALE));
                    canvas.clear(Color4f::new(1.0, 1.0, 1.0, 1.0));
                    let value = json!({"todos": todos, "todo": todo});
                    let input = user_input(take(&mut events), &mut mouse_position)
                        .fonts(&mut fonts)
                        .value(value);
                    let output = component.update(input);
                    for element in output.elements {
                        draw_element(canvas, &element, &fonts, &mut images);
                    }
                    for call in output.calls {
                        println!("CALL {call:?}");
                        match call.describe() {
                            ("append", [todo]) => {
                                todos.push(todo.as_str().unwrap().to_string());
                            }
                            ("edit", [value]) => {
                                todo = value.as_str().unwrap().to_string();
                            }
                            ("remove", [todo]) => {
                                todos.retain(|record| record != todo);
                            }
                            _ => {}
                        };
                        window.request_redraw();
                    }
                });
            }
            event => {
                events.push(event);
                window.request_redraw();
            }
        },

        _ => {}
    };
    events_loop
        .run(move |event, window_target| autoreleasepool(|| event_handler(event)))
        .expect("run() failed");
}

fn draw_element(canvas: &Canvas, element: &Element, fonts: &FontSystem, images: &mut ImageSystem) {
    let rect = Rect::from_xywh(
        element.layout.location.x,
        element.layout.location.y,
        element.layout.size.width,
        element.layout.size.height,
    );
    canvas.draw_rect(rect, &Paint::new(color(element.background.color), None));
    draw_borders(canvas, element);
    if let Some(text) = element.text.as_ref() {
        let paint = Paint::new(color(element.color), None);
        canvas.draw_text_align(
            text,
            Point::new(
                element.layout.location.x,
                element.layout.location.y + fonts.offset_y(text, &element.text_style),
            ),
            &fonts.get_font(element.text_style.font_size),
            &paint,
            Align::Left,
        );
    }
    if let Some(image) = element.background.image.as_ref() {
        if let Some(image) = images.load(&image) {
            let paint = Paint::default();
            canvas.draw_image_rect(image, None, &rect, &paint);
        }
    }
}

fn draw_borders(canvas: &Canvas, element: &Element) -> Result<(), String> {
    let borders = &element.borders;
    let layout = &element.layout;
    let x = layout.location.x as f32;
    let y = layout.location.y as f32;
    let w = layout.size.width as f32;
    let h = layout.size.height as f32;
    let paint = &mut Paint::new(Color4f::new(0.0, 0.0, 0.0, 0.0), None);

    if let Some(border) = borders.top.as_ref() {
        paint.set_color4f(color(border.color), None);
        paint.set_stroke_width(border.width);
        canvas.draw_line((x, y), (x + w, y), paint);
    }
    if let Some(border) = borders.bottom.as_ref() {
        paint.set_color4f(color(border.color), None);
        paint.set_stroke_width(border.width);
        canvas.draw_line((x, y + h), (x + w, y + h), paint);
    }
    if let Some(border) = borders.left.as_ref() {
        paint.set_color4f(color(border.color), None);
        paint.set_stroke_width(border.width);
        canvas.draw_line((x, y), (x, y + h), paint);
    }
    if let Some(border) = borders.left.as_ref() {
        paint.set_color4f(color(border.color), None);
        paint.set_stroke_width(border.width);
        canvas.draw_line((x + w, y), (x + w, y + h), paint);
    }
    Ok(())
}

fn color(value: Rgba) -> Color4f {
    let [r, g, b, a] = value;
    Color4f::new(
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    )
}

struct ImageSystem {
    images: HashMap<String, Image>,
}

impl ImageSystem {
    fn new() -> Self {
        Self {
            images: HashMap::new(),
        }
    }

    pub fn load(&mut self, absolute_path: &str) -> Option<&Image> {
        if !self.images.contains_key(absolute_path) {
            let data =
                Data::from_filename(absolute_path).expect("failed to read image data from file");
            let image = Image::from_encoded(data).expect("failed to load iamge from data");
            self.images.insert(absolute_path.to_string(), image);
        }
        self.images.get(absolute_path)
    }
}

struct FontSystem {
    typeface: Typeface,
}

impl FontSystem {
    fn new() -> Self {
        let font_mgr: FontMgr = FontMgr::new();
        let font_data = fs::read("../shared/Roboto/Roboto-Regular.ttf")
            .expect("failed to read font data from file");
        let typeface = font_mgr
            .new_from_data(&font_data, None)
            .expect("failed to load font");
        Self { typeface }
    }

    fn get_font(&self, size: f32) -> Font {
        Font::from_typeface(self.typeface.clone(), Some(size))
    }

    pub fn offset_y(&self, text: &str, style: &TextStyle) -> f32 {
        let (_, rect) = self.get_font(style.font_size).measure_text(text, None);
        -rect.top
    }
}

impl Fonts for FontSystem {
    fn measure(&mut self, text: &str, style: &TextStyle, _max_width: Option<f32>) -> [f32; 2] {
        let (_, rect) = self.get_font(style.font_size).measure_text(text, None);
        [rect.width(), rect.height()]
    }
}

fn user_input<'f>(events: Vec<WindowEvent>, mouse_position: &mut [f32; 2]) -> Input<'f> {
    let mut characters = vec![];
    let mut keys_pressed = vec![];
    let mut buttons_down = vec![];

    for event in events {
        match event {
            // Event::KeyDown { keycode, .. } => if let Some(keycode) = keycode {
            //     keys_down.push(map_keycode(keycode))
            // },
            WindowEvent::CursorMoved { position, .. } => {
                let px = position.x as f32 / WINDOW_SCALE;
                let py = position.y as f32 / WINDOW_SCALE;
                *mouse_position = [px, py];
            }
            WindowEvent::MouseInput { button, state, .. } => match state {
                ElementState::Pressed => match button {
                    MouseButton::Left => buttons_down.push(0),
                    MouseButton::Right => buttons_down.push(1),
                    _ => {}
                },
                ElementState::Released => {}
            },
            WindowEvent::KeyboardInput { event, .. } => {
                if let Some(text) = event.text {
                    for char in text.chars() {
                        println!("CHAR {char:?}");
                        if char != '\u{8}' {
                            characters.push(char);
                        }
                    }
                }
                match event.state {
                    ElementState::Pressed => {}
                    ElementState::Released => keys_pressed.push(map_keycode(event.physical_key)),
                }
            }
            _ => {}
        }
    }

    Input::new()
        .viewport([WINDOW_WIDTH, WINDOW_HEIGHT])
        .mouse_position(*mouse_position)
        .mouse_buttons_down(buttons_down)
        // .mouse_buttons_up(buttons_up)
        .characters(characters)
        // .keys_down(keys_down)
        // .keys_up(keys_up)
        .keys_pressed(keys_pressed)
}

pub fn map_keycode(key: PhysicalKey) -> Keys {
    let code = match key {
        PhysicalKey::Code(code) => code,
        PhysicalKey::Unidentified(_) => return Keys::Unknown,
    };
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
        KeyCode::ArrowUp => Keys::ArrowUp,
        KeyCode::ArrowDown => Keys::ArrowDown,
        KeyCode::ArrowLeft => Keys::ArrowLeft,
        KeyCode::ArrowRight => Keys::ArrowRight,
        KeyCode::End => Keys::End,
        KeyCode::Home => Keys::Home,
        KeyCode::PageDown => Keys::PageDown,
        KeyCode::PageUp => Keys::PageUp,
        // Modifier keys
        KeyCode::AltLeft => Keys::Alt,
        KeyCode::AltRight => Keys::Alt,
        KeyCode::CapsLock => Keys::CapsLock,
        KeyCode::ControlLeft => Keys::Ctrl,
        KeyCode::ControlRight => Keys::Ctrl,
        KeyCode::ShiftLeft => Keys::Shift,
        KeyCode::ShiftRight => Keys::Shift,
        _ => Keys::Unknown,
    }
}
