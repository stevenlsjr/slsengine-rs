#![allow(dead_code)]

use cgmath::prelude::*;
use cgmath::{Decomposed, Quaternion};
use failure;
use hibitset::BitSet;
use specs::prelude::*;
use specs::Entity;

use slsengine::math::{Mat4, Vec3};
#[cfg(feature = "backend-vulkan")]
use slsengine::renderer::backend_vk;
use slsengine::renderer::components::{MeshComponent, TransformComponent};
use slsengine::{
    application::*,
    game::*,
    renderer::*,
    sdl_platform::{self, Platform},
};
use slsengine::{
    renderer::backend_gl::{self, gl_renderer::GlRenderer},
    sdl_platform::{OpenGLVersion, OpenGLVersion::GL45},
};



fn setup_gl() -> Result<Application<backend_gl::GlRenderer>, failure::Error> {
    let platform = sdl_platform::platform()
        .with_opengl(OpenGLVersion::GL41)
        .build_gl()?;
    let renderer = backend_gl::GlRenderer::new(&platform.window)?;
    let main_loop = MainLoopState::new();
    let world = WorldManager::new(&renderer);
    Ok(Application {
        platform,
        renderer,
        main_loop,
        world_manager: world,
    })
}
#[derive(Default, Debug)]
struct SpawnEntitiesSystem {
    is_run: bool,
}
use cgmath::*;
impl<'a> System<'a> for SpawnEntitiesSystem {
    type SystemData = (Entities<'a>, WriteStorage<'a, TransformComponent>);
    fn run(&mut self, (entities, ref mut xform_storage): Self::SystemData) {
        use specs::world::EntityResBuilder;
        let mut entity_vec = Vec::<Entity>::new();
        if !self.is_run {
            self.is_run = true;
            for i in 0..4 {
                for j in 0..4 {
                    let position = cgmath::vec3(i as f32, j as f32, 0.0);
                    let mut xform = TransformComponent::default();
                    xform.transform.disp = position;
                    let e = entities
                        .build_entity()
                        .with(xform, xform_storage)
                        .build();
                    entity_vec.push(e);
                }
            }
            println!("{:?}", entity_vec);
        }
    }
}

fn main() -> Result<(), i32> {
    let mut app = setup_gl().map_err(|e| {
        eprintln!("app error: {}", e);
        1
    })?;
   

    let mut entities: Vec<Entity> = Vec::new();
    {
        let world = app.world_mut();

        let transform: Decomposed<Vec3, Quaternion<f32>> = Decomposed::one();

        entities.push(e);
    }

    app.run()
}
