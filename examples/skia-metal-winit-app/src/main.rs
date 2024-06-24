use core_graphics_types::geometry::CGSize;
use objc::rc::autoreleasepool;
use skia_safe::{Canvas, Color4f, Paint, Point, Rect, Size};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

use crate::metal::create_metal_layer;

mod metal;

fn main() {
    let size = LogicalSize::new(800, 600);

    let events_loop = EventLoop::new().expect("Failed to create event loop");

    let window = WindowBuilder::new()
        .with_inner_size(size)
        .with_title("Skia+Metal+Winit bumaga example".to_string())
        .build(&events_loop)
        .unwrap();

    let mut metal = create_metal_layer(&window);

    events_loop
        .run(move |event, window_target| {
            autoreleasepool(|| {
                if let Event::WindowEvent { event, .. } = event {
                    match event {
                        WindowEvent::CloseRequested => window_target.exit(),
                        WindowEvent::Resized(size) => {
                            metal.layer.set_drawable_size(CGSize::new(
                                size.width as f64,
                                size.height as f64,
                            ));
                            window.request_redraw()
                        }
                        WindowEvent::RedrawRequested => {
                            metal.redraw(|canvas| {
                                draw(canvas);
                            });
                        }
                        _ => (),
                    }
                }
            });
        })
        .expect("run() failed");
}

fn draw(canvas: &Canvas) {
    let canvas_size = Size::from(canvas.base_layer_size());

    canvas.clear(Color4f::new(1.0, 1.0, 1.0, 1.0));

    let rect_size = canvas_size / 2.0;
    let rect = Rect::from_point_and_size(
        Point::new(
            (canvas_size.width - rect_size.width) / 2.0,
            (canvas_size.height - rect_size.height) / 2.0,
        ),
        rect_size,
    );
    canvas.draw_rect(rect, &Paint::new(Color4f::new(0.0, 0.0, 1.0, 1.0), None));
}
