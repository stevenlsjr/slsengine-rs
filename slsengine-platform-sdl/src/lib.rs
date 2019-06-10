#[macro_use]
extern crate serde_derive;

use failure;
use log::*;
pub use sdl2;
use sdl2::video::{GLContext, Window, WindowBuilder};
use sdl2::Sdl;
use sdl2::VideoSubsystem;
///
/// Module handling creation of SDL and graphics api contexts
use std::cell::{Ref, RefCell};
use std::fmt;
use std::ptr;
use std::rc::Rc;
use log::*;

pub mod config;



use config::PlatformConfig;

///
/// Container for SDL-level resources, such as subsystems and the
/// primary window.
pub struct Platform {
    pub window: Window,
    pub video_subsystem: VideoSubsystem,
    pub sdl_context: Sdl,
    pub gl_context: Option<GLContext>,
    pub event_pump: Rc<RefCell<sdl2::EventPump>>,
    pub render_backend: RenderBackend,
}

impl Platform {}

impl fmt::Debug for Platform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let addr = {
            let ptr = self.window.raw() as *const _;
            ptr as usize
        };
        write!(f, r#"Platform {{window: 0x{:x}, ..}}"#, addr)
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let title = self.window.title();
        write!(f, r#"Platform for {}"#, title)
    }
}

pub type PlatformResult<T> = Result<T, failure::Error>;

/// Constructs a PlatformBuilder struct
pub fn platform() -> PlatformBuilder {
    PlatformBuilder::new()
}

#[derive(Debug, PartialEq, Clone)]
pub enum OpenGLVersion {
    GL32,
    GL33,
    GL41,
    GL45,
    GL46,
}

#[derive(Debug, PartialEq, Clone)]
pub enum RenderBackend {
    OpenGL = 0,
    Vulkan,
    Undefined,
}

/// Defines hooks for platform creation lifecycle.
/// Provides both method hooks for constructing platform windows,
/// as well as a structure for storing side-effects of platform creation
pub trait PlatformBuilderHooks {
    ///
    /// given the platform builder instance and video subsystem, constructs a SDL window.
    /// Called after sdl runtime and video subsystem creation
    ///
    fn build_window(
        &self,
        platform_builder: &PlatformBuilder,
        video_subsystem: &VideoSubsystem,
    ) -> PlatformResult<Window>;
}

///
/// Builder patter for Platform
pub struct PlatformBuilder {
    config: PlatformConfig,
    opengl_version: Option<(u32, u32)>,
    render_backend: RenderBackend,
}

impl PlatformBuilder {
    fn new() -> PlatformBuilder {
        PlatformBuilder {
            opengl_version: None,
            config: PlatformConfig::default(),
            render_backend: RenderBackend::Undefined,
        }
    }

    pub fn with_opengl(
        &mut self,
        version: OpenGLVersion,
    ) -> &mut PlatformBuilder {
        self.opengl_version = match version {
            OpenGLVersion::GL32 => Some((3, 2)),
            OpenGLVersion::GL33 => Some((3, 3)),
            OpenGLVersion::GL41 => Some((4, 1)),
            OpenGLVersion::GL45 => Some((4, 5)),
            OpenGLVersion::GL46 => Some((4, 6)),
        };
        self.render_backend = RenderBackend::OpenGL;
        self
    }
    #[inline]
    pub fn with_config(&mut self, config: PlatformConfig) -> &mut Self {
        self.config = config;
        self
    }
    #[inline]
    pub fn config(&self) -> &PlatformConfig {
        &self.config
    }

    pub fn build<H: PlatformBuilderHooks>(
        &self,
        hooks: &H,
    ) -> PlatformResult<Platform> {
        let sdl_context = sdl2::init().map_err(&failure::err_msg)?;
        let video_subsystem = sdl_context.video().map_err(&failure::err_msg)?;

        let window = hooks.build_window(self, &video_subsystem)?;
        {
            let event_pump =
                sdl_context.event_pump().map_err(&failure::err_msg)?;

            let render_backend = self.render_backend.clone();
            Ok(Platform {
                gl_context: None,
                window,
                video_subsystem,
                sdl_context,
                event_pump: Rc::new(RefCell::new(event_pump)),
                render_backend,
            })
        }
    }

}

/// creates WindowBuilder from PlatformBuilder parameters
pub fn make_window_builder(
    platform_builder: &PlatformBuilder,
    video_subsystem: &VideoSubsystem,
) -> WindowBuilder {
    let config = &platform_builder.config;
    let title = &config.window_title;
    let (width, height) = config.window_size;

    let mut wb = video_subsystem.window(title, width, height);

    if config.allow_highdpi {
        wb.allow_highdpi();
    }

    if config.fullscreen {
        wb.fullscreen();
    }
    wb
}
