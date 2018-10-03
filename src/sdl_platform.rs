use sdl2::{Sdl, VideoSubsystem};
use sdl2::video::{GLContext, Window};
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
///
/// Module handling creation of SDL and graphics api contexts

use super::failure;
use super::get_error_desc;
use super::sdl2;

pub enum PlatformError {}

///
/// Container for SDL-level resources, such as subsystems and the
/// primary window.
pub struct Platform {
    pub window: Window,
    pub video_subsystem: VideoSubsystem,
    pub sdl_context: Sdl,
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

pub type PlatformResult<T> = Result<T, String>;

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

///
/// Builder patter for Platform
pub struct PlatformBuilder {
    window_size: (u32, u32),
    window_title: String,
    opengl_version: Option<(u32, u32)>,
    render_backend: RenderBackend,
}

impl PlatformBuilder {
    fn new() -> PlatformBuilder {
        PlatformBuilder {
            window_size: (640, 480),
            window_title: "Window".to_string(),
            opengl_version: None,
            render_backend: RenderBackend::Undefined,
        }
    }

    pub fn with_opengl(&mut self, version: OpenGLVersion) -> &mut PlatformBuilder {
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

    pub fn with_vulkan(&mut self) -> &mut PlatformBuilder {
        self.render_backend = RenderBackend::Vulkan;
        self
    }

    pub fn with_window_size(
        &mut self,
        width: u32,
        height: u32,
    ) -> &mut PlatformBuilder {
        self.window_size = (width, height);
        self
    }

    pub fn with_window_title(&mut self, title: &str) -> &mut PlatformBuilder {
        self.window_title = title.to_string();
        self
    }

    pub fn build(&self) -> PlatformResult<Platform> {
        use sdl2::video::GLProfile;

        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;
        if self.render_backend == RenderBackend::OpenGL {
            let (major, minor) = self.opengl_version.unwrap_or((4, 1));
            let gl_attr = video_subsystem.gl_attr();
            gl_attr.set_context_profile(GLProfile::Core);
            gl_attr.set_context_flags().debug().set();
            gl_attr.set_context_version(major as u8, minor as u8);
            gl_attr.set_multisample_buffers(1);
            gl_attr.set_multisample_samples(4);
        }

        let title = self.window_title.as_str();
        let (width, height) = self.window_size;
        let window = video_subsystem
            .window(title, width, height)
            .resizable()
            .opengl()
            .build()
            .map_err(get_error_desc)?;
        {
            let event_pump = sdl_context.event_pump()?;

            let render_backend = self.render_backend.clone();
            Ok(Platform {
                window: window,
                video_subsystem,
                sdl_context,
                event_pump: Rc::new(RefCell::new(event_pump)),
                render_backend,
            })
        }
    }
}

pub fn load_opengl(platform: &Platform) -> Result<GLContext, String> {
    use super::renderer::gl;
    match platform.render_backend {
        RenderBackend::OpenGL => {}
        _ => { return Err(format!("backend {:?} is not openGL", platform.render_backend)); }
    }
    let ctx = platform.window.gl_create_context()?;

    gl::load_with(|s| platform.video_subsystem.gl_get_proc_address(s) as *const _);


    platform.window.gl_make_current(&ctx)?;
    Ok(ctx)
}
