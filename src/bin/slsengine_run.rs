#![allow(dead_code)]

#[cfg(feature = "backend-vulkan")]
use slsengine::renderer::backend_vk;
use slsengine::{
    game::*,
    renderer::*,
    sdl_platform::{self, Platform},
};

use failure;
#[cfg(feature = "backend-gl")]
use slsengine::renderer::backend_gl;

struct App<R: Renderer> {
    platform: Platform,
    renderer: R,
    main_loop: MainLoopState,
    world: EntityWorld<R>,
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

    Ok(())
}
