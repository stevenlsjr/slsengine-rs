#![allow(dead_code)]

use hibitset::BitSet;
#[cfg(feature = "backend-vulkan")]
use slsengine::renderer::backend_vk;
use slsengine::{
    game::*,
    renderer::*,
    sdl_platform::{self, Platform},
};

use failure;
#[cfg(feature = "backend-gl")]
use slsengine::{
    renderer::backend_gl::{self, gl_renderer::GlRenderer},
    sdl_platform::{OpenGLVersion, OpenGLVersion::GL45},
};

use slsengine::game;
use slsengine::game::resource::DeltaTime;
#[cfg(feature = "backend-gl")]
use specs::prelude::*;

struct App<R: Renderer> {
    platform: Platform,
    renderer: R,
    main_loop: MainLoopState,
    world: WorldManager<R>,
}

#[cfg(feature = "backend-vulkan")]
fn setup_vk() -> Result<App<backend_vk::VulkanRenderer>, failure::Error> {
    let platform =
        sdl_platform::platform().build(&backend_vk::VulkanPlatformHooks)?;
    let renderer = backend_vk::VulkanRenderer::new(&platform.window)?;
    let main_loop = MainLoopState::new();
    let world = WorldManager::new(&renderer);
    Ok(App {
        platform,
        renderer,
        main_loop,
        world,
    })
}

#[cfg(feature = "backend-gl")]
fn setup_gl() -> Result<App<backend_gl::GlRenderer>, failure::Error> {
    let (platform, gl) = sdl_platform::platform()
        .with_opengl(OpenGLVersion::GL41)
        .build_gl()?;
    let renderer = backend_gl::GlRenderer::new(&platform.window)?;
    let main_loop = MainLoopState::new();
    let world = WorldManager::new(&renderer);
    Ok(App {
        platform,
        renderer,
        main_loop,
        world,
    })
}

fn main() -> Result<(), i32> {
    let mut app = setup_gl().map_err(|e| {
        eprintln!("app error: {}", e);
        1
    })?;

    app.main_loop.start();
    let mut update_dispatch = DispatcherBuilder::new().build();
    while app.main_loop.is_running() {
        {
            let App {
                ref mut main_loop,
                ref renderer,
                ref mut world,
                ref mut platform,
                ..
            } = app;
            main_loop.handle_events(
                &mut platform.window,
                &mut platform.event_pump,
                renderer,
                world,
            );
        }
        let frame = app.main_loop.tick_frame();
        let entity_world = app.world.world();
        {
            let mut dt = entity_world.write_resource::<DeltaTime>();
            dt.0 = frame.delta;
        }
    }

    Ok(())
}
