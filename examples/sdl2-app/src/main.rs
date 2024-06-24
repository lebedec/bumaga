use std::collections::{HashMap, HashSet};
use std::process;

use sdl2::event::Event;
use sdl2::EventPump;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Canvas, Texture, TextureCreator, WindowCanvas};
use sdl2::ttf::{Font, Sdl2TtfContext};
use serde_json::json;

use bumaga::{Component, CssColor, Element, Fonts, Input, Keys, TextStyle};

fn main() {
    run().unwrap()
}

const WINDOW_W: u32 = 800;

const WINDOW_H: u32 = 600;

fn run() -> Result<(), String> {
    let system = sdl2::init()?;
    let video = system.video()?;
    let ttf = sdl2::ttf::init().map_err(|error| error.to_string())?;

    let mut fonts = FontSystem::new(&ttf);

    let window = video
        .window("SDL2 bumaga example", WINDOW_W, WINDOW_H)
        .position_centered()
        .opengl()
        .build()
        .map_err(|error| error.to_string())?;

    let mut canvas = window
        .into_canvas()
        .build()
        .map_err(|error| error.to_string())?;

    let mut component = Component::compile_files("../shared/index.html", "../shared/style.css");
    let todos = [
        "learn bumaga documentation",
        "create UI using HTML",
        "implement engine",
    ];
    let mut todos = HashSet::from(todos.map(&str::to_string));
    let mut todo = "Enter a todo".to_string();
    loop {
        canvas.set_draw_color(Color::RGBA(195, 217, 255, 255));
        canvas.clear();
        let value = json!({"todos": todos, "todo": todo});
        let input = user_input(system.event_pump()?)
            .fonts(&mut fonts)
            .value(value);
        let output = component.update(input);
        for element in output.elements {
            draw_element(&mut canvas, &element, &mut fonts)?;
        }
        for call in output.calls {
            println!("CALL {call:?}");
            match call.describe() {
                ("append", [todo]) => {
                    todos.insert(todo.as_str().unwrap().to_string());
                }
                ("edit", [value]) => {
                    todo = value.as_str().unwrap().to_string();
                }
                ("remove", [todo]) => {
                    todos.remove(todo.as_str().unwrap());
                }
                _ => {}
            };
        }

        canvas.present();
    }
}

fn draw_element(
    canvas: &mut WindowCanvas,
    element: &Element,
    fonts: &mut FontSystem,
) -> Result<(), String> {
    let texture_creator = canvas.texture_creator();
    let color = color(&element.background.color);
    if color.a > 0 {
        canvas.set_draw_color(color);
        canvas.set_blend_mode(BlendMode::Blend);
        canvas.fill_rect(Rect::new(
            element.layout.location.x as i32,
            element.layout.location.y as i32,
            element.layout.size.width as u32,
            element.layout.size.height as u32,
        ))?;
    }
    draw_borders(canvas, element)?;
    if let Some(text) = element.text.as_ref() {
        let font = fonts.get_font(element.text_style.font_size as u16);
        let color = &element.color;
        let color = Color::RGBA(color.red, color.green, color.blue, color.alpha);
        let surface = font
            .render(text)
            .blended(color)
            .map_err(|error| error.to_string())?;
        let texture =
            Texture::from_surface(&surface, &texture_creator).map_err(|error| error.to_string())?;
        canvas.copy(
            &texture,
            None,
            Some(Rect::new(
                element.layout.location.x as i32,
                element.layout.location.y as i32,
                surface.width(),
                surface.height(),
            )),
        )?;
    }
    Ok(())
}

fn draw_borders(canvas: &mut WindowCanvas, element: &Element) -> Result<(), String> {
    let borders = &element.borders;
    let layout = &element.layout;
    let x = layout.location.x as i16;
    let y = layout.location.y as i16;
    let w = layout.size.width as i16;
    let h = layout.size.height as i16;

    if let Some(border) = borders.top.as_ref() {
        canvas.line(x, y, x + w, y, color(&border.color))?;
    }
    if let Some(border) = borders.bottom.as_ref() {
        canvas.line(x, y + h, x + w, y + h, color(&border.color))?;
    }
    if let Some(border) = borders.left.as_ref() {
        canvas.line(x, y, x, y + h, color(&border.color))?;
    }
    if let Some(border) = borders.left.as_ref() {
        canvas.line(x + w, y, x + w, y + h, color(&border.color))?;
    }
    Ok(())
}

fn color(css: &CssColor) -> Color {
    match css {
        CssColor::RGBA(color) => Color::RGBA(color.red, color.green, color.blue, color.alpha),
        _ => Color::RED,
    }
}

struct FontSystem<'ttf> {
    fonts: HashMap<u16, Font<'ttf, 'static>>,
    ttf: &'ttf Sdl2TtfContext,
}

impl<'ttf> FontSystem<'ttf> {
    pub fn new(ttf: &'ttf Sdl2TtfContext) -> Self {
        Self {
            fonts: Default::default(),
            ttf,
        }
    }

    pub fn get_font(&mut self, size: u16) -> &Font {
        if !self.fonts.contains_key(&size) {
            let font = self
                .ttf
                .load_font("../shared/Roboto/Roboto-Regular.ttf", size)
                .unwrap();
            self.fonts.insert(size, font);
        }
        self.fonts.get(&size).unwrap()
    }
}

impl<'ttf> Fonts for FontSystem<'ttf> {
    fn measure(&mut self, text: &str, style: &TextStyle, _max_width: Option<f32>) -> [f32; 2] {
        // NOTE: SDL_ttf does not support width constraint measurement,
        // only single line text will be rendered correctly
        let font = self.get_font(style.font_size as u16);
        let (width, height) = font.size_of(text).unwrap();
        [width as f32, height as f32]
    }
}

fn user_input<'f>(mut events: EventPump) -> Input<'f> {
    let mut characters = vec![];
    let mut keys_pressed = vec![];
    let mut buttons_down = vec![];
    for event in events.poll_iter() {
        match event {
            // Event::KeyDown { keycode, .. } => if let Some(keycode) = keycode {
            //     keys_down.push(map_keycode(keycode))
            // },
            Event::MouseButtonDown { mouse_btn, .. } => match mouse_btn {
                MouseButton::Left => buttons_down.push(0),
                MouseButton::Right => buttons_down.push(1),
                _ => {}
            },
            Event::KeyUp { keycode, .. } => {
                if let Some(keycode) = keycode {
                    keys_pressed.push(map_keycode(keycode))
                }
            }
            Event::TextInput { text, .. } => characters = text.chars().collect(),
            Event::Quit { .. } => process::exit(0),
            _ => {}
        }
    }

    let mouse_state = events.mouse_state();
    let mouse_position = [mouse_state.x() as f32, mouse_state.y() as f32];
    Input::new()
        .viewport([WINDOW_W as f32, WINDOW_H as f32])
        .mouse_position(mouse_position)
        .mouse_buttons_down(buttons_down)
        // .mouse_buttons_up(buttons_up)
        .characters(characters)
        // .keys_down(keys_down)
        // .keys_up(keys_up)
        .keys_pressed(keys_pressed)
}

pub fn map_keycode(code: Keycode) -> Keys {
    match code {
        // UI keys
        Keycode::Escape => Keys::Escape,
        // Editing keys
        Keycode::Backspace => Keys::Backspace,
        Keycode::Delete => Keys::Delete,
        Keycode::Insert => Keys::Insert,
        // Whitespace keys
        Keycode::Return => Keys::Enter,
        Keycode::Tab => Keys::Tab,
        // Navigation keys
        Keycode::Up => Keys::ArrowUp,
        Keycode::Down => Keys::ArrowDown,
        Keycode::Left => Keys::ArrowLeft,
        Keycode::Right => Keys::ArrowRight,
        Keycode::End => Keys::End,
        Keycode::Home => Keys::Home,
        Keycode::PageDown => Keys::PageDown,
        Keycode::PageUp => Keys::PageUp,
        // Modifier keys
        Keycode::LAlt => Keys::Alt,
        Keycode::RAlt => Keys::Alt,
        Keycode::CapsLock => Keys::CapsLock,
        Keycode::LCtrl => Keys::Ctrl,
        Keycode::RCtrl => Keys::Ctrl,
        Keycode::LShift => Keys::Shift,
        Keycode::RShift => Keys::Shift,
        _ => Keys::Unknown,
    }
}
