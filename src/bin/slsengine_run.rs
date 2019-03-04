#![allow(dead_code)]

use hibitset::BitSet;
#[cfg(feature = "backend-vulkan")]
use slsengine::renderer::backend_vk;
use slsengine::{
    game::*,
    game::component_store::NullComponentStore,
    renderer::*,
    sdl_platform::{self, Platform},
};

use failure;
#[cfg(feature = "backend-gl")]
use slsengine::renderer::backend_gl;
use slsengine::game;

struct App<R: Renderer> {
    platform: Platform,
    renderer: R,
    main_loop: MainLoopState,
    world: EntityWorld<R, NullComponentStore>,
}

#[cfg(feature = "backend-vulkan")]
fn setup() -> Result<App<backend_vk::VulkanRenderer>, failure::Error> {
    let platform =
        sdl_platform::platform().build(&backend_vk::VulkanPlatformHooks)?;
    let renderer = backend_vk::VulkanRenderer::new(&platform.window)?;
    let main_loop = MainLoopState::new();
    let world = EntityWorld::new(&renderer);
    Ok(App {
        platform,
        renderer,
        main_loop,
        world,
    })
}
#[cfg(all(not(feature = "backend-vulkan"), feature = "backend-gl"))]
fn setup() -> Result<App<backend_gl::GlRenderer>, failure::Error> {
    let platform =
        sdl_platform::platform().build(&backend_vk::VulkanPlatformHooks)?;
    let renderer = backend_vk::VulkanRenderer::new(&platform.window)?;
    let main_loop = MainLoopState::new();
    let world = EntityWorld::new(&renderer);
    Ok(App {
        platform,
        renderer,
        main_loop,
        world,
    })
}

#[derive(Debug)]
struct Point(u32, u32);
impl Component for Point {}

fn main() -> Result<(), i32> {
    let mut app = setup().map_err(|e| {
        eprintln!("setup failed: {:?}", e);
        1
    })?;
    let entity = {
        let world: &mut EntityWorld<_> = &mut app.world;
        world.components.register::<Point>();
        let entity = world.components.alloc_entity();
        let points = world.components.get_components::<Point>().unwrap();
        {
            let mut lock = points.write().unwrap();
            lock[*entity] = Some(Point(0, 20));
        }
        world.components.calc_mask(entity);
        entity
    };
    let mask: &BitSet = app.world.components.entity_mask(entity);
    let point_mask = app.world.components.component_mask::<Point>();
    // assert!(mask.contains(point_mask));
    let id_table = app.world.components.id_table();
    assert_eq!(point_mask, id_table.get::<Point>().unwrap());
    assert_eq!(mask.clone().into_iter().count(), 1);

    dbg!(("point mask", point_mask));
    for i in mask.clone().into_iter() {
        dbg!(i);
    }
    use std::any::TypeId;
    dbg!(TypeId::of::<Point>());

    Ok(())
}
