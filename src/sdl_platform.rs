use gl;
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
    
    let  mut wb = video_subsystem.window(title, width, height);

    if cfg!(target_os="macos") {
        println!("enabling highdpi display");
        wb.allow_highdpi();
    }
    wb
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
        #[cfg(feature = "gl-debug-output")]
        {
            gl_attr.set_context_flags().debug().set();
        }

        wb.opengl();
        wb.resizable();
        let window = wb.build().map_err(|e| e.to_string())?;

        let gl_ctx = load_opengl(&window, video_subsystem)?;
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
    eprintln!("------------------------------");
    eprintln!(
        "Error {} (0x{:x}):",
        message.to_str().unwrap_or("unknown error"),
        id
    );
    match source {
        gl::DEBUG_SOURCE_API => eprint!("Source: DEBUG_SOURCE_API\t"),
        gl::DEBUG_SOURCE_WINDOW_SYSTEM => {
            eprint!("Source: DEBUG_SOURCE_WINDOW_SYSTEM\t")
        }
        gl::DEBUG_SOURCE_SHADER_COMPILER => {
            eprint!("Source: DEBUG_SOURCE_SHADER_COMPILER\t")
        }
        gl::DEBUG_SOURCE_THIRD_PARTY => {
            eprint!("Source: DEBUG_SOURCE_THIRD_PARTY\t")
        }
        gl::DEBUG_SOURCE_APPLICATION => {
            eprint!("Source: DEBUG_SOURCE_APPLICATION\t")
        }
        gl::DEBUG_SOURCE_OTHER => eprint!("Source: DEBUG_SOURCE_OTHER\t"),
        other => eprint!("Source: unknown:0x{:x};\t", other),
    };
    match err_type {
        gl::DEBUG_TYPE_ERROR => eprint!("Type: DEBUG_TYPE_ERROR\t"),
        gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => {
            eprint!("Type: DEBUG_SOURCE_OTHER\t")
        }
        gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => {
            eprint!("Type: DEBUG_TYPE_UNDEFINED_BEHAVIOR\t")
        }
        gl::DEBUG_TYPE_PORTABILITY => eprint!("Type: DEBUG_TYPE_PORTABILITY\t"),
        gl::DEBUG_TYPE_PERFORMANCE => eprint!("Type: DEBUG_TYPE_PORTABILITY\t"),
        gl::DEBUG_TYPE_MARKER => eprint!("Type: DEBUG_TYPE_PORTABILITY\t"),
        gl::DEBUG_TYPE_PUSH_GROUP => eprint!("Type: DEBUG_TYPE_PUSH_GROUP\t"),
        gl::DEBUG_TYPE_POP_GROUP => eprint!("Type: DEBUG_TYPE_POP_GROUP\t"),

        gl::DEBUG_TYPE_OTHER => eprint!("Type: DEBUG_TYPE_OTHER\t"),
        other => eprint!("Type: unknown 0x{:x}\t", other),
    };

    match severity {
        gl::DEBUG_SEVERITY_HIGH => eprint!("Severity: high\n"),
        gl::DEBUG_SEVERITY_MEDIUM => eprint!("Severity: medium\n"),
        gl::DEBUG_SEVERITY_LOW => eprint!("Severity: low\n"),
        other => eprint!("Severity: Unknown 0x{:x}\t", other),
    };
    eprintln!("\n------------------------------");
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
