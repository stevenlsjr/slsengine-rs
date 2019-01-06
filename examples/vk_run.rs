use failure;
use image::{ImageBuffer, Rgba};
use slsengine::{
    self,
    game::{main_loop::*, world::*},
    renderer::backend_vk::*,
    renderer::*,
    sdl_platform::*,
};
use std::sync::Arc;
use vulkano::{
    buffer::*, command_buffer::*, format::*, framebuffer::*, image::*,
    impl_vertex, instance::debug::*, pipeline::*, single_pass_renderpass,
    sync::*,
};

#[derive(Debug, Clone, Copy)]
struct SimpleVertex {
    position: [f32; 2],
}
impl_vertex!(SimpleVertex, position);

fn triangle_verts() -> Vec<SimpleVertex> {
    let vertex1 = SimpleVertex {
        position: [-0.5, -0.5],
    };
    let vertex2 = SimpleVertex {
        position: [0.0, 0.5],
    };
    let vertex3 = SimpleVertex {
        position: [0.5, -0.25],
    };
    vec![vertex1, vertex2, vertex3]
}

fn main() {
    env_logger::init();
    let (width, height) = (1024, 1024);

    let platform = platform().build(&VulkanPlatformHooks).unwrap();

    let mut r = VulkanRenderer::new(&platform.window).unwrap();
    let VulkanRenderer {
        ref instance,
        ref device,
        queues: ref q,
        ..
    } = r;
    let dbg_callback = DebugCallback::new(
        instance,
        MessageTypes {
            error: true,
            warning: true,
            performance_warning: true,
            information: true,
            debug: true,
        },
        |msg| {
            eprintln!("vulkan {:?} message, layer {} callback: '{}'", msg.ty, msg.layer_prefix, msg.description);
        },
    );

    let image = StorageImage::new(
        r.device.clone(),
        Dimensions::Dim2d { width, height },
        Format::R8G8B8A8Unorm,
        Some(q.graphics_queue.family()),
    )
    .unwrap();

    let vertex_array = {
        CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            triangle_verts().into_iter(),
        )
        .unwrap()
    };
    {
        let Platform {
            ref window,
            ref event_pump,
            ..
        } = &platform;
        let mut main_loop = MainLoopState::new();
        let mut world = EntityWorld::new(&r);
        main_loop.start();
        while main_loop.is_running() {
            main_loop.handle_events(window, event_pump, &r, &mut world);
            let FrameTick { delta, .. } = main_loop.tick_frame();
            r.draw_frame(window);
        }
    }
}

fn save_img_buffer(buf: &CpuAccessibleBuffer<[u8]>, size: (u32, u32)) {
    use std::path::*;
    let content = buf.read().unwrap();
    let path = Path::new(env!("OUT_DIR")).join("vk_run.png");
    let (width, height) = size;
    let img = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, &content[..])
        .unwrap();
    img.save(&path).unwrap();
    println!("image saved to {}", path.to_string_lossy())
}
