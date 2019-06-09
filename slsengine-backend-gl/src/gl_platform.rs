use std::cell::{Ref, RefCell};
use std::ptr;

use log::*;

use slsengine_platform_sdl::{
    make_window_builder, Platform, PlatformBuilder, PlatformBuilderHooks,
    PlatformResult,
};
use slsengine_platform_sdl::sdl2::{self, video::*, VideoSubsystem};

pub struct GlPlatformBuilder {
    pub gl_ctx: RefCell<Option<GLContext>>,
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
        let mut wb = make_window_builder(platform_builder, video_subsystem);

        let gl_attr = video_subsystem.gl_attr();

        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_context_version(4, 1);
        let n_samples = platform_builder.config().antialiasing.n_samples();
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

        let gl_ctx =
            load_opengl(&window, video_subsystem).map_err(&failure::err_msg)?;
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
        gl::DEBUG_TYPE_PORTABILITY => warn!("Type: DEBUG_TYPE_PORTABILITY\t"),
        gl::DEBUG_TYPE_PERFORMANCE => warn!("Type: DEBUG_TYPE_PORTABILITY\t"),
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

///
/// Extension trait for a Platformbuilder to
/// build OpenGL context with Sdl platform
pub trait BuildGlPlatform {
    fn build_gl(&self) -> PlatformResult<Platform>;
}

impl BuildGlPlatform for PlatformBuilder {
    fn build_gl(&self) -> PlatformResult<Platform> {
        let pb = GlPlatformBuilder::new();
        let mut platform = self.build(&pb)?;
        platform.gl_context = pb.gl_ctx.replace(None);
        Ok(platform)
    }
}
