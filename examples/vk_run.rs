use failure;
use image::{ImageBuffer, Rgba};
use slsengine::{
    self,
    game::{
        self, built_in_components::*, component::*, main_loop::*, system::*,
        world::*, *,
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
    game: &mut EntityWorld<VulkanRenderer>,
) {
    use genmesh::generators::*;
    use slsengine::game::{
        built_in_components::*, component::ComponentMask, resource::MeshHandle,
    };
    let helmet_mesh = {
        use gltf::*;
        use slsengine::renderer::model::*;

        let model =
            Model::from_gltf("assets/models/Corset.glb").unwrap();
        let mut mesh = model.meshes[0].mesh.clone();
        let scale = Matrix4::from_scale(10.0);

        for mut v in &mut mesh.vertices {
            let pos =  Vector3::from(v.position).extend(1.0);
            v.position = (scale * pos).truncate().into();
        }

        VkMesh::new(renderer, mesh).unwrap()
    };
    let sphere_mesh =
        VkMesh::new(renderer, Mesh::from_genmesh(IcoSphere::subdivide(4)))
            .unwrap();

<<<<<<< HEAD
=======
    let mesh_handle_a = MeshHandle(0);
    let mesh_handle_b = MeshHandle(1);
    game.resources.meshes.insert(mesh_handle_a, helmet_mesh);
    game.resources.meshes.insert(mesh_handle_b, sphere_mesh);

    let n = 4;
    let entities: Vec<_> = (0..=n)
        .map(|i| {
            let e = game.components.alloc_entity();
            game.components.transforms[*e] = {
                let mut xform = TransformComponent::default();
                let x_pos = (n as f32 / 2.0 - i as f32) * 2.2;
                xform.transform.disp = vec3(x_pos, 0.0, 0.0);
                Some(xform)
            };
            let mesh = if i % 2 == 0 {
                mesh_handle_a
            } else {
                mesh_handle_b
            };
            game.components.meshes[*e] = Some(MeshComponent { mesh });
            game.components.calc_mask(e);
            debug_assert!(game.components.masks[*e]
                .unwrap()
                .contains(ComponentMask::TRANSFORM | ComponentMask::MESH));
            e
        })
        .collect();
>>>>>>> origin/feature_entity_index
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
            r.draw_frame(window, &world);
        }
    }
}
