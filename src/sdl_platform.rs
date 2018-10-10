use super::sdl2;
pub use sdl2::video::{GLContext, Window, WindowBuilder};
use sdl2::Sdl;
pub use sdl2::VideoSubsystem;
///
/// Module handling creation of SDL and graphics api contexts
use std::cell::{Ref, RefCell};
use std::fmt;
use std::rc::Rc;

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
    pub window_size: (u32, u32),
    pub window_title: String,
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

    pub fn build<H: PlatformBuilderHooks>(
        &self,
        hooks: &H,
    ) -> PlatformResult<Platform> {
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;

        let window = hooks.build_window(self, &video_subsystem)?;
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

    pub fn build_gl(&self) -> PlatformResult<(Platform, GlPlatformBuilder)> {
        let pb = GlPlatformBuilder::new();
        let platform = self.build(&pb)?;
        Ok((platform, pb))
    }
}

/// creates WindowBuilder from PlatformBuilder parameters
pub fn make_window_builder(
    platform_builder: &PlatformBuilder,
    video_subsystem: &VideoSubsystem,
) -> WindowBuilder {
    let title = &platform_builder.window_title;
    let (width, height) = platform_builder.window_size;
    video_subsystem.window(title, width, height)
}

pub struct GlPlatformBuilder {
    gl_ctx: RefCell<Option<GLContext>>,
}
impl GlPlatformBuilder {
    fn new() -> GlPlatformBuilder {
        GlPlatformBuilder {
            gl_ctx: RefCell::new(None),
        }
    }

    pub fn gl_ctx(&self) -> Ref<Option<GLContext>> {
        self.gl_ctx.borrow()
    }
}
impl PlatformBuilderHooks for GlPlatformBuilder {
    fn build_window(
        &self,
        platform_builder: &PlatformBuilder,
        video_subsystem: &VideoSubsystem,
    ) -> PlatformResult<Window> {
        use sdl2::video::GLProfile;
        let mut wb = make_window_builder(platform_builder, video_subsystem);
        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_context_version(4, 1);
        wb.opengl();
        let window = wb.build().map_err(|e| e.to_string())?;

        let gl_ctx = load_opengl(&window, video_subsystem)?;
        self.gl_ctx.replace(Some(gl_ctx));

        Ok(window)
    }
}

pub fn load_opengl(
    window: &Window,
    video_subsystem: &VideoSubsystem,
) -> Result<GLContext, String> {
    use super::renderer::gl;

    let ctx = window.gl_create_context()?;

    gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const _);

    window.gl_make_current(&ctx)?;
    Ok(ctx)
}
