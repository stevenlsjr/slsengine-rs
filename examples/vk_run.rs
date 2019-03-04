use failure;
use image::{ImageBuffer, Rgba};
use slsengine::{
    self,
    game::{
        self, built_in_components::*, component::*,
        component_stores::NullComponentStore, main_loop::*, system::*, world::*,
    },
    renderer::backend_vk::*,
    renderer::*,
    sdl_platform::*,
};
use slsengine_entityalloc::*;
use std::sync::Arc;
use vulkano::{
    buffer::*, command_buffer::*, format::*, framebuffer::*, image::*,
    instance::debug::*, pipeline::*, single_pass_renderpass, sync::*,
};

use cgmath::*;

fn setup_game(
    renderer: &VulkanRenderer,
    game: &mut EntityWorld<VulkanRenderer, NullComponentStore>,
) {
    use genmesh::generators::*;
    use slsengine::game::{
        built_in_components::*, component::ComponentMask, resource::MeshHandle,
    };
    let helmet_mesh = {
        use slsengine::renderer::model::*;

        let model = Model::from_gltf("assets/models/Corset.glb").unwrap();
        let mut mesh = model.meshes[0].mesh.clone();
        let scale = Matrix4::from_scale(10.0);

        for mut v in &mut mesh.vertices {
            let pos = Vector3::from(v.position).extend(1.0);
            v.position = (scale * pos).truncate().into();
        }

        VkMesh::new(renderer, mesh).unwrap()
    };
    let sphere_mesh =
        VkMesh::new(renderer, Mesh::from_genmesh(IcoSphere::subdivide(4)))
            .unwrap();
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
        let mut world = EntityWorld::new(&r, NullComponentStore);
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
            r.draw_frame(window, &world);
        }
    }
}
