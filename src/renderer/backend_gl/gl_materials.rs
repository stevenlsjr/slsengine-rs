use super::objects::{BufferObject, ObjectError, SingleBuffer, UboBindings};
use gl;
use renderer::material::Material;

/// Material ubo representation shared by shader memory
/// uses GLSL std140 layout
#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct MaterialBufferLayout {
    pub albedo_factor: [f32; 4],
    pub roughness_factor: f32,
    pub metallic_factor: f32,
    pub emissive: [f32; 4],
    _padding_end: [u8; 8],
}

impl MaterialBufferLayout {
    pub fn from_material<T>(mat: &Material<T>) -> Self {
        MaterialBufferLayout {
            albedo_factor: mat.albedo_factor.into(),
            roughness_factor: mat.roughness_factor,
            metallic_factor: mat.metallic_factor,
            emissive: mat.emissive_factor.extend(0.0).into(),
            ..MaterialBufferLayout::default()
        }
    }
}

impl Default for MaterialBufferLayout {
    fn default() -> Self {
        use std::mem::zeroed;
        unsafe { zeroed() }
    }
}

#[derive(Debug)]
pub struct MaterialUbo {
    ubo: SingleBuffer,
}

const MATERIAL_UBO_SIZE: usize = 48;

impl MaterialUbo {
    pub fn new() -> Result<Self, ObjectError> {
        use std::ptr;
        let ubo = SingleBuffer::new()?;
        unsafe {
            ubo.buffer().bind(gl::UNIFORM_BUFFER);
            let _id = ubo.0;
            gl::BufferData(
                gl::UNIFORM_BUFFER,
                MATERIAL_UBO_SIZE as isize,
                ptr::null(),
                gl::STATIC_DRAW,
            );
        }
        Ok(MaterialUbo { ubo })
    }

    #[inline]
    pub fn buffer(&self) -> BufferObject {
        self.ubo.buffer()
    }

    pub fn setup_binding(&self, program: &super::gl_renderer::Program) {
        use std::ffi::CStr;
        let block_name = CStr::from_bytes_with_nul(b"Material\0").unwrap();

        unsafe {
            let index =
                gl::GetUniformBlockIndex(program.id(), block_name.as_ptr());
            gl::UniformBlockBinding(
                program.id(),
                index,
                UboBindings::Material as u32,
            );
            gl::BindBufferBase(
                gl::UNIFORM_BUFFER,
                UboBindings::Material as u32,
                self.buffer().id(),
            );
        }
    }

    pub fn set_material<T>(
        &self,
        material: &::renderer::material::Material<T>,
    ) -> Result<(), super::GlErrors> {
        use super::gl_renderer::{drain_error_stack, dump_errors};
        use gl::types::*;
        let _buffer = MaterialBufferLayout::from_material(material);
        drain_error_stack();
        unsafe {
            self.buffer().bind(gl::UNIFORM_BUFFER);
            let mapped_buffer: *mut MaterialBufferLayout =
                gl::MapBuffer(gl::UNIFORM_BUFFER, gl::WRITE_ONLY) as *mut _;
            dump_errors()?;

            *mapped_buffer = MaterialBufferLayout::from_material(material);

            gl::UnmapBuffer(gl::UNIFORM_BUFFER);
            dump_errors()?;
            self.buffer().unbind(gl::UNIFORM_BUFFER);
        }
        Ok(())
    }

    pub fn bind_to_program(
        &self,
        program: &super::gl_renderer::Program,
    ) -> Result<(), super::GlErrors> {
        self.setup_binding(program);

        Ok(())
    }
}

