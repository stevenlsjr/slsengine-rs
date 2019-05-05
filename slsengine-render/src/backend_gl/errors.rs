use failure;
use log::*;
use std::fmt;

#[derive(Fail)]
#[fail(display = "OpenGL errors: {:?}", errors)]
pub struct GlErrors {
    pub errors: Vec<gl::types::GLenum>,
}
impl fmt::Debug for GlErrors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::fmt::Write;
        f.write_str("GlErrors: {[")?;
        for (i, error) in self.errors.iter().enumerate() {
            write!(f, "0x{:x}", error)?;
            if i < self.errors.len() - 1 {
                f.write_char(',')?;
            }
        }
        f.write_str("]}")
    }
}

#[derive(Fail, Debug)]
pub enum RendererError {
    #[fail(display = "RenderRc lifecycle failed: {}", reason)]
    Lifecycle { reason: String },
    #[fail(display = "Failed to construct Geometry: {}", reason)]
    GeometryFailure { reason: String },
    #[fail(display = "ShaderError: {}", _0)]
    ShaderError(ShaderError),
}

#[derive(Fail, Debug)]
pub enum ShaderError {
    #[fail(display = "invalid shader type: 0x{:X}", shader_type)]
    InvalidType { shader_type: u32 },

    #[fail(display = "Unable to construct shader, reason: {:?}", reason)]
    ApiFailure { reason: String },

    #[fail(display = "Shader compilation failed, log: {:?}", info_log)]
    CompileFailure { info_log: String },

    #[fail(display = "Link failed, reason: {:?}", reason)]
    LinkFailure { reason: String },
    #[fail(display = "Could not bind uniform, name: {}, {}", name, msg)]
    UniformBindFailure { name: String, msg: String },
}

/// Silently drain errors from OpenGL error stack.
/// This is neccessary if you wish to
/// record errors accumulated over any given openGL operation(s)
pub fn drain_error_stack() {
    loop {
        let err = unsafe { gl::GetError() };
        if err == gl::NO_ERROR {
            break;
        }
    }
}

/// Returns a Result where the `Err` case contains
/// a list of OpenGl errors recorded. Call `drain_error_stack`
/// before the operation you wish to check for errors
pub fn dump_errors() -> Result<(), GlErrors> {
    let mut errors = Vec::new();

    loop {
        let err = unsafe { gl::GetError() };
        if err == gl::NO_ERROR {
            break;
        }
        errors.push(err);
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(GlErrors { errors })
    }
}

pub fn debug_error_stack(file: &str, line: u32) {
    loop {
        let err = unsafe { gl::GetError() };
        if err == gl::NO_ERROR {
            break;
        }
        error!("{}, {}, GL error: 0x{:X}", file, line, err)
    }
}
