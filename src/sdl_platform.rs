use failure;
#[cfg(feature = "backend-gl")]
use gl;
use log::*;
use sdl2;
use sdl2::video::{GLContext, Window, WindowBuilder};
use sdl2::Sdl;
use sdl2::VideoSubsystem;
///
/// Module handling creation of SDL and graphics api contexts
use std::cell::{Ref, RefCell};
use std::fmt;
use std::ptr;
use std::rc::Rc;

use crate::config::PlatformConfig;

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
                window,
                video_subsystem,
                sdl_context,
                event_pump: Rc::new(RefCell::new(event_pump)),
                render_backend,
            })
        }
    }
    #[cfg(feature = "backend-gl")]
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
#[cfg(feature = "backend-gl")]
pub mod gl_platform {
    use super::*;
    use sdl2::video::GLContext;

    pub struct GlPlatformBuilder {
        gl_ctx: RefCell<Option<GLContext>>,
    }

    impl GlPlatformBuilder {
        pub fn new() -> GlPlatformBuilder {
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
            let n_samples = platform_builder.config.antialiasing.n_samples();
            if n_samples > 0 {
                gl_attr.set_multisample_buffers(1);
                gl_attr.set_multisample_samples(n_samples as u8);
            }
            #[cfg(feature = "gl-debug-output")]
            {
                gl_attr.set_context_flags().debug().set();
            }

            wb.opengl();
            wb.resizable();
            let window = wb.build()?;

            let gl_ctx = load_opengl(&window, video_subsystem)
                .map_err(&failure::err_msg)?;
            {
                self.gl_ctx.replace(Some(gl_ctx));
            }

            Ok(window)
        }
    }

    #[allow(unused_variables, dead_code)]
    extern "system" fn gl_debug_output(
        source: gl::types::GLenum,
        err_type: gl::types::GLenum,
        id: gl::types::GLuint,
        severity: gl::types::GLenum,
        length: gl::types::GLsizei,
        _message: *const gl::types::GLchar,
        user_param: *mut gl::types::GLvoid,
    ) {
        use std::ffi::CStr;
        let message = unsafe { CStr::from_ptr(_message) };
        if severity != gl::DEBUG_SEVERITY_LOW
            || severity != gl::DEBUG_SEVERITY_MEDIUM
            || severity != gl::DEBUG_SEVERITY_HIGH
        {
            return;
        }
        warn!(
            "------------------------------\nError {} (0x{:x}):",
            message.to_str().unwrap_or("unknown error"),
            id
        );

        match source {
            gl::DEBUG_SOURCE_API => warn!("Source: DEBUG_SOURCE_API\t"),
            gl::DEBUG_SOURCE_WINDOW_SYSTEM => {
                warn!("Source: DEBUG_SOURCE_WINDOW_SYSTEM\t")
            }
            gl::DEBUG_SOURCE_SHADER_COMPILER => {
                warn!("Source: DEBUG_SOURCE_SHADER_COMPILER\t")
            }
            gl::DEBUG_SOURCE_THIRD_PARTY => {
                warn!("Source: DEBUG_SOURCE_THIRD_PARTY\t")
            }
            gl::DEBUG_SOURCE_APPLICATION => {
                warn!("Source: DEBUG_SOURCE_APPLICATION\t")
            }
            gl::DEBUG_SOURCE_OTHER => warn!("Source: DEBUG_SOURCE_OTHER\t"),
            other => warn!("Source: unknown:0x{:x};\t", other),
        };
        match err_type {
            gl::DEBUG_TYPE_ERROR => warn!("Type: DEBUG_TYPE_ERROR\t"),
            gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => {
                warn!("Type: DEBUG_SOURCE_OTHER\t")
            }
            gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => {
                warn!("Type: DEBUG_TYPE_UNDEFINED_BEHAVIOR\t")
            }
            gl::DEBUG_TYPE_PORTABILITY => {
                warn!("Type: DEBUG_TYPE_PORTABILITY\t")
            }
            gl::DEBUG_TYPE_PERFORMANCE => {
                warn!("Type: DEBUG_TYPE_PORTABILITY\t")
            }
            gl::DEBUG_TYPE_MARKER => warn!("Type: DEBUG_TYPE_PORTABILITY\t"),
            gl::DEBUG_TYPE_PUSH_GROUP => warn!("Type: DEBUG_TYPE_PUSH_GROUP\t"),
            gl::DEBUG_TYPE_POP_GROUP => warn!("Type: DEBUG_TYPE_POP_GROUP\t"),

            gl::DEBUG_TYPE_OTHER => warn!("Type: DEBUG_TYPE_OTHER\t"),
            other => warn!("Type: unknown 0x{:x}\t", other),
        };

        match severity {
            gl::DEBUG_SEVERITY_HIGH => warn!("Severity: high\n"),
            gl::DEBUG_SEVERITY_MEDIUM => warn!("Severity: medium\n"),
            gl::DEBUG_SEVERITY_LOW => warn!("Severity: low\n"),
            other => warn!("Severity: Unknown 0x{:x}\t", other),
        };
        warn!("\n------------------------------");
    }
    pub fn load_opengl(
        window: &Window,
        video_subsystem: &VideoSubsystem,
    ) -> Result<GLContext, String> {
        let ctx = window.gl_create_context()?;

        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const _);
        // setup opengl debug context
        {
            let mut flags: i32 = 0;
            unsafe { gl::GetIntegerv(gl::CONTEXT_FLAGS, &mut flags) };
            if 0 != (flags & gl::CONTEXT_FLAG_DEBUG_BIT as i32) {
                unsafe {
                    gl::Enable(gl::DEBUG_OUTPUT);
                    gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
                    gl::DebugMessageCallback(gl_debug_output, ptr::null());
                    gl::DebugMessageControl(
                        gl::DONT_CARE,
                        gl::DONT_CARE,
                        gl::DEBUG_SEVERITY_LOW,
                        0,
                        ptr::null(),
                        gl::TRUE,
                    );
                }
            }
        }

        window.gl_make_current(&ctx)?;
        Ok(ctx)
    }

}

#[cfg(feature = "backend-gl")]
pub use self::gl_platform::{load_opengl, GlPlatformBuilder};
