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

    let platform = platform().build(&VulkanPlatformHooks).unwrap();

    let r = VulkanRenderer::new(&platform.window).unwrap();
    let VulkanRenderer {
        ref instance,
        ref device,
        queues: ref q,
        ..
    } = r;
    let _dbg_callback = DebugCallback::errors_and_warnings(
        instance,
        |msg| {
            eprintln!(
                "vulkan {:?} message, layer {} callback: '{}'",
                msg.ty, msg.layer_prefix, msg.description
            );
        },
    );

    let staging_vb = {
        CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            triangle_verts().into_iter(),
        )
        .unwrap()
    };
    let vertex_array = {
        let vbo = DeviceLocalBuffer::<[Vertex]>::array(
            device.clone(),
            3,
            BufferUsage::all(),
            vec![q.graphics_queue.family()],
        )
        .unwrap();
        let cb = AutoCommandBufferBuilder::new(
            device.clone(),
            q.graphics_queue.family(),
        )
        .unwrap()
        .copy_buffer(staging_vb.clone(), vbo.clone())
        .unwrap()
        .build()
        .unwrap();

        cb.execute(q.graphics_queue.clone())
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap()
            .wait(None)
            .unwrap();
        vbo
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
            r.draw_frame(window, &world, vertex_array.clone());
        }
    }
}
