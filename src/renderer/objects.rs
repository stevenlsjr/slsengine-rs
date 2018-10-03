/**
 *  objects.rs: Managed OpenGL buffer, texture, and vertex objects
 **/
extern crate failure;
use super::gl;
use super::gl::types::*;

#[derive(Fail, Debug)]
pub enum ObjectError {
    #[fail(display = "Could not create object: {:?}", reason)]
    ObjectCreationFailure { reason: String },
}

pub enum BufferObjectTarget {
    ArrayBuffer = gl::ARRAY_BUFFER as isize,
    AtomicCounterBuffer = gl::ATOMIC_COUNTER_BUFFER as isize,
    CopyReadBuffer = gl::COPY_READ_BUFFER as isize,
    CopyWriteBuffer = gl::COPY_WRITE_BUFFER as isize,
    DispatchIndirectBuffer = gl::DISPATCH_INDIRECT_BUFFER as isize,
    DrawIndirectBuffer = gl::DRAW_INDIRECT_BUFFER as isize,
    ElementArrayBuffer = gl::ELEMENT_ARRAY_BUFFER as isize,
    PixelPackBuffer = gl::PIXEL_PACK_BUFFER as isize,
    PixelUnpackBuffer = gl::PIXEL_UNPACK_BUFFER as isize,
    QueryBuffer = gl::QUERY_BUFFER as isize,
    ShaderStorageBuffer = gl::SHADER_STORAGE_BUFFER as isize,
    TextureBuffer = gl::TEXTURE_BUFFER as isize,
    TransformFeedbackBuffer = gl::TRANSFORM_FEEDBACK_BUFFER as isize,
    UniformBuffer = gl::UNIFORM_BUFFER as isize,
}

///
/// Representation of a single Buffer object Handle
#[derive(Debug)]
pub struct BufferObject(u32);

impl BufferObject {
    /// returns buffer handle
    pub fn id(&self) -> u32 {
        self.0
    }

    /// binds buffer to `target` in openGL context
    pub unsafe fn bind(&self, target: GLenum) {
        gl::BindBuffer(target, self.0);
    }
}

#[derive(Debug)]
pub struct VertexArrayObject(u32);

impl VertexArrayObject {
    /// returns array handle
    pub fn id(&self) -> u32 {
        self.0
    }

    /// binds vertx array to the global
    /// opengl context
    pub unsafe fn bind(&self) {
        gl::BindVertexArray(self.0)
    }
}

#[derive(Debug)]
pub struct BufferObjects {
    objects: Vec<u32>,
}

impl BufferObjects {
    pub fn new(n_objects: usize) -> BufferObjects {
        let mut objects: Vec<u32> = vec![0; n_objects];
        unsafe {
            let ptr: *mut u32 = objects.as_mut_ptr();
            gl::GenBuffers(n_objects as i32, ptr);
        }

        BufferObjects { objects: objects }
    }
}

impl Drop for BufferObjects {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(
                self.objects.len() as i32,
                self.objects.as_mut_ptr(),
            );
        }
        self.objects.clear();
    }
}

#[derive(Debug)]
pub struct MeshBuffers {
    pub vertex_buffer: BufferObject,
    pub index_buffer: BufferObject,
    pub vertex_array: VertexArrayObject,
}

impl MeshBuffers {
    pub fn new() -> Result<MeshBuffers, ObjectError> {
        let mut buffers = [0u32, 0];
        let mut vao = 0u32;

        unsafe {
            super::drain_error_stack();
            gl::GenBuffers(2, buffers.as_mut_ptr());
            let mut errors: Vec<gl::types::GLenum> = Vec::new();
            super::dump_errors(&mut errors);

            gl::GenVertexArrays(1, &mut vao);
        }

        Ok(MeshBuffers {
            vertex_buffer: BufferObject(buffers[0]),
            index_buffer: BufferObject(buffers[1]),
            vertex_array: VertexArrayObject(vao),
        })
    }
}

impl Drop for MeshBuffers {
    fn drop(&mut self) {
        let mut buff_objs = [self.index_buffer.0, self.vertex_buffer.0];
        let mut vao = self.vertex_array.0;
        unsafe {
            gl::DeleteBuffers(2, buff_objs.as_mut_ptr());
            gl::DeleteVertexArrays(1, &mut vao);
        }
    }
}
