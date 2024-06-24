use cocoa::{appkit::NSView, base::id as cocoa_id};
use core_graphics_types::geometry::CGSize;
use foreign_types_shared::{ForeignType, ForeignTypeRef};
use metal_rs::{CommandQueue, Device, MetalLayer, MTLPixelFormat};
use objc::runtime::YES;
use raw_window_handle::HasWindowHandle;
use skia_safe::{Canvas, ColorType, gpu, scalar, Size};
use skia_safe::gpu::{DirectContext, mtl, SurfaceOrigin};
use skia_safe::gpu::mtl::BackendContext;
use winit::window::Window;

pub struct MetalImpl {
    pub layer: MetalLayer,
    pub device: Device,
    pub command_queue: CommandQueue,
    // skia
    pub backend: BackendContext,
    pub context: DirectContext,
}

impl MetalImpl {
    pub fn redraw<F>(&mut self, mut draw: F)
    where
        F: FnMut(&Canvas),
    {
        if let Some(drawable) = self.layer.next_drawable() {
            let drawable_size = {
                let size = self.layer.drawable_size();
                Size::new(size.width as scalar, size.height as scalar)
            };

            let mut surface = unsafe {
                let texture_info =
                    mtl::TextureInfo::new(drawable.texture().as_ptr() as mtl::Handle);

                let backend_render_target = gpu::backend_render_targets::make_mtl(
                    (drawable_size.width as i32, drawable_size.height as i32),
                    &texture_info,
                );

                gpu::surfaces::wrap_backend_render_target(
                    &mut self.context,
                    &backend_render_target,
                    SurfaceOrigin::TopLeft,
                    ColorType::BGRA8888,
                    None,
                    None,
                )
                .unwrap()
            };

            draw(surface.canvas());

            self.context.flush_and_submit();
            drop(surface);

            let command_buffer = self.command_queue.new_command_buffer();
            command_buffer.present_drawable(drawable);
            command_buffer.commit();
        }
    }
}

pub fn create_metal_layer(window: &Window) -> MetalImpl {
    let device = Device::system_default().expect("no device found");

    let command_queue = device.new_command_queue();

    let backend = unsafe {
        BackendContext::new(
            device.as_ptr() as mtl::Handle,
            command_queue.as_ptr() as mtl::Handle,
        )
    };

    let context = gpu::direct_contexts::make_metal(&backend, None).unwrap();

    let window_handle = window
        .window_handle()
        .expect("Failed to retrieve a window handle");

    let raw_window_handle = window_handle.as_raw();

    let draw_size = window.inner_size();
    let layer = MetalLayer::new();
    layer.set_device(&device);
    layer.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
    layer.set_presents_with_transaction(false);
    // Disabling this option allows Skia's Blend Mode to work.
    // More about: https://developer.apple.com/documentation/quartzcore/cametallayer/1478168-framebufferonly
    layer.set_framebuffer_only(false);

    unsafe {
        let view = match raw_window_handle {
            raw_window_handle::RawWindowHandle::AppKit(appkit) => appkit.ns_view.as_ptr(),
            _ => panic!("Wrong window handle type"),
        } as cocoa_id;
        view.setWantsLayer(YES);
        view.setLayer(layer.as_ref() as *const _ as _);
    }
    layer.set_drawable_size(CGSize::new(draw_size.width as f64, draw_size.height as f64));

    MetalImpl {
        layer,
        device,
        command_queue,
        backend,
        context,
    }
}
