#![allow(dead_code)]

use hibitset::BitSet;

#[cfg(feature = "backend-vulkan")]
use slsengine::renderer::backend_vk;
use slsengine::{
    game::*,
    renderer::*,
    application::*,
    sdl_platform::{self, Platform},
};

use failure;
#[cfg(feature = "backend-gl")]
use slsengine::{
    renderer::backend_gl::{self, gl_renderer::GlRenderer},
    sdl_platform::{OpenGLVersion, OpenGLVersion::GL45},
};

#[cfg(feature = "backend-vulkan")]
fn setup_vk() -> Result<Application<backend_vk::VulkanRenderer>, failure::Error> {
    let platform =
        sdl_platform::platform().build(&backend_vk::VulkanPlatformHooks)?;
    let renderer = backend_vk::VulkanRenderer::new(&platform.window)?;
    let main_loop = MainLoopState::new();
    let world_manager = WorldManager::new(&renderer);
    Ok(Application {
        platform,
        renderer,
        main_loop,
        world_manager,
    })
}

#[cfg(feature = "backend-gl")]
fn setup_gl() -> Result<Application<backend_gl::GlRenderer>, failure::Error> {
    let (platform, gl) = sdl_platform::platform()
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

fn main() -> Result<(), i32> {

    let mut app = setup_gl().map_err(|e| {
        eprintln!("app error: {}", e);
        1
    })?;

    app.run()

}
