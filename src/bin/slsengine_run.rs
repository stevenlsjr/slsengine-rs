#![allow(dead_code)]

use hibitset::BitSet;
#[cfg(feature = "backend-vulkan")]
use slsengine::renderer::backend_vk;
use slsengine::{
    game::*,
    game::component_stores::NullComponentStore,
    renderer::*,
    sdl_platform::{self, Platform},
};

use failure;
#[cfg(feature = "backend-gl")]
use slsengine::renderer::backend_gl;
#[cfg(feature = "backend-gl")]
use slsengine::renderer::backend_gl::gl_renderer::GlRenderer;

use slsengine::game;
use slsengine::sdl_platform::OpenGLVersion::GL45;
use slsengine::sdl_platform::OpenGLVersion;

struct App<R: Renderer> {
    platform: Platform,
    renderer: R,
    main_loop: MainLoopState,
    world: EntityWorld<R, NullComponentStore>,
}

#[cfg(feature = "backend-vulkan")]
fn setup_vk() -> Result<App<backend_vk::VulkanRenderer>, failure::Error> {
    let platform =
        sdl_platform::platform().build(&backend_vk::VulkanPlatformHooks)?;
    let renderer = backend_vk::VulkanRenderer::new(&platform.window)?;
    let main_loop = MainLoopState::new();
    let world = EntityWorld::new(&renderer, NullComponentStore);
    Ok(App {
        platform,
        renderer,
        main_loop,
        world,
    })
}

#[cfg(feature = "backend-gl")]
fn setup_gl() -> Result<App<backend_gl::GlRenderer>, failure::Error> {
    let (platform, gl) =
        sdl_platform::platform().with_opengl(OpenGLVersion::GL41).build_gl()?;
    let renderer = backend_gl::GlRenderer::new(&platform.window)?;
    let main_loop = MainLoopState::new();
    let world = EntityWorld::new(&renderer, NullComponentStore);
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

fn run_app<R: Renderer>(app: App<R>) -> Result<(), failure::Error> {
    Ok(())
}

#[cfg(feature="backend-vulkan")]
fn main() -> Result<(), i32> {

    setup_vk().and_then(&run_app).map_err(|e| {
        eprintln!("app error: {}", e);
       1
    })
}

#[cfg(all(not(feature="backend-vulkan"), feature="backend-gl"))]
fn main() -> Result<(), i32> {

    setup_vk().and_then(&run_app).map_err(|e| {
        eprintln!("app error: {}", e);
        1
    })
}
