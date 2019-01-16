use failure;
use image::{ImageBuffer, Rgba};
use slsengine::{
    self,
    game::{self, main_loop::*, world::*, *},
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

fn triangle_verts() -> Vec<Vertex> {
    let vertex1 = Vertex {
        position: [-0.5, -0.5, 0.0],
        ..Vertex::default()
    };
    let vertex2 = Vertex {
        position: [0.0, 0.5, 1.0],
        ..Vertex::default()
    };
    let vertex3 = Vertex {
        position: [0.5, -0.25, 0.0],
        ..Vertex::default()
    };
    vec![vertex1, vertex2, vertex3]
}

fn main() {
    env_logger::init();
    let (width, height) = (1024, 1024);

    let platform = platform().build(&VulkanPlatformHooks).unwrap();

    let r = VulkanRenderer::new(&platform.window).unwrap();
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
            eprintln!(
                "vulkan {:?} message, layer {} callback: '{}'",
                msg.ty, msg.layer_prefix, msg.description
            );
        },
    );

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
        } = platform;
        let mut main_loop = MainLoopState::new();
        let mut world = EntityWorld::new(&r);
        main_loop.start();
        while main_loop.is_running() {
            main_loop.handle_events(window, &event_pump, &r, &mut world);
            if !main_loop.is_running() {
                break;
            }
            let FrameTick { delta, .. } = main_loop.tick_frame();
            {
                let ep = event_pump.borrow();

                world.update(delta, game::InputSources::from_event_pump(&ep));
            }
            r.draw_frame(window, &world, &vertex_array);
        }
    }
}
