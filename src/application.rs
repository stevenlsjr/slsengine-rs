use std::any::Any;
use std::fmt::{Debug, Display};

use sdl2::event::EventType::AppDidEnterBackground;
use specs::prelude::*;

use crate::game::main_loop::FrameTick;
use crate::game::WorldManager;
use crate::MainLoopState;
///!
///! Application lifecycle manager

use crate::renderer::Renderer;
use crate::sdl_platform::{Platform, RenderBackend};
use crate::sdl_platform::RenderBackend::Undefined;

pub struct ApplicationBuilder {
    preferred_backend: RenderBackend,
}

impl ApplicationBuilder {
    pub fn new() -> Self {
        ApplicationBuilder { preferred_backend: Undefined }
    }

    fn build<R: Renderer>(self) -> Result<Application<R>, failure::Error> {
        unimplemented!()
    }
}


pub fn application() -> ApplicationBuilder {
    ApplicationBuilder::new()
}


#[derive(Debug)]
pub struct Application<R: Renderer> {
    pub platform: Platform,
    pub renderer: R,
    pub main_loop: MainLoopState,
    pub world_manager: WorldManager,
}

impl<R: Renderer> Application<R> {
    #[inline]
    pub fn world(&self) -> &World {
        self.world_manager.world()
    }

    #[inline]
    pub fn world_mut(&mut self) -> &mut World {
        self.world_manager.world_mut()
    }
    pub fn run(mut self) -> Result<(), i32> {
        self.setup().map_err(|e| {
            eprintln!("app startup error! {:?}", e);
            -1
        })?;
        while self.main_loop.is_running() {
            {
                let mut frame = self.world_manager.world_mut().write_resource::<FrameTick>();
                *frame = self.main_loop.tick_frame();
            }
            {
                self.main_loop.handle_events(&self.platform.window,
                                             &self.platform.event_pump,
                                             &self.renderer,
                                             &mut self.world_manager);
            }

            self.renderer.render_system(&self.platform.window,self.world_manager.world_mut());


        }

        Ok(())
    }
    fn setup(&mut self) -> Result<(), failure::Error> {
        self.main_loop.start();
        let frame = self.main_loop.tick_frame();
        self.world_manager.world_mut().add_resource(frame);
        Ok(())
    }
}

#[derive(Fail, Debug)]
pub enum AppError {
    #[fail(display = "App error: '{}'", _0)]
    Other(failure::Error),
}


impl AppError {
    pub fn from_message<D: Display + Debug + Send + Sync + Sized + 'static>(
        message: D,
    ) -> AppError {
        AppError::Other(failure::err_msg(message))
    }
}
