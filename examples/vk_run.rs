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

use cgmath::*;


fn setup_game(
    renderer: &VulkanRenderer,
    game: &mut EntityWorld<VulkanRenderer>,
) {
    use genmesh::{generators::*};
    use slsengine::game::{
        built_in_components::*, component::ComponentMask, resource::MeshHandle,
    };
    let icosphere = IcoSphere::subdivide(2);

    let vk_mesh = {
        let mesh = Mesh {
            vertices: icosphere
                .shared_vertex_iter()
                .map(|v| Vertex {
                    position: v.pos.into(),
                    normal: v.normal.into(),
                    ..Vertex::default()
                })
                .collect(),
            indices: icosphere
                .indexed_polygon_iter()
                .flat_map(|t| vec![t.x as u32, t.y as u32, t.z as u32])
                .collect(),
        };
        VkMesh::new(renderer, mesh).unwrap()
    };
    let mesh_handle = MeshHandle(0);
    game.resources.meshes.insert(mesh_handle, vk_mesh);

    let triangles: Vec<_> = (0..4)
        .map(|i| {
            let e = game.components.alloc_entity();
            game.components.transforms[*e] = {
                let mut xform = TransformComponent::default();
                xform.transform.disp = vec3((i as f32) as f32, 0.0, 0.0);
                Some(xform)
            };
            game.components.calc_mask(e);
            e
        })
        .collect();
}

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
    let _dbg_callback = DebugCallback::errors_and_warnings(instance, |msg| {
        eprintln!(
            "vulkan {:?} message, layer {} callback: '{}'",
            msg.ty, msg.layer_prefix, msg.description
        );
    });

    {
        use slsengine::game::resource::*;
        let Platform {
            ref window,
            ref event_pump,
            ..
        } = platform;
        let mut main_loop = MainLoopState::new();
        let mut world = EntityWorld::new(&r);
        setup_game(&r, &mut world);

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
            r.draw_frame(
                window,
                &world,
                &world.resources.meshes[&MeshHandle(0)],
            );
        }
    }
}
