/**
 *  objects.rs: Managed OpenGL buffer, texture, and vertex objects
 **/
extern crate gl;

extern crate failure;

#[derive(Fail, Debug)]
pub enum ObjectError {
    #[fail(display = "Could not create object: {:?}", reason)]
    ObjectCreationFailure { reason: String },
}

///
/// Representation of a single Buffer object Handle
#[derive(Debug)]
pub struct BufferObject(u32);

#[derive(Debug)]
pub struct VertexArrayObject(u32);

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
    pub vertex_array_object: VertexArrayObject,
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
            vertex_array_object: VertexArrayObject(vao),
        })
    }
}

impl Drop for MeshBuffers {
    fn drop(&mut self) {
        let mut buff_objs = [self.index_buffer.0, self.vertex_buffer.0];
        let mut vao = self.vertex_array_object.0;
        unsafe {
            gl::DeleteBuffers(2, buff_objs.as_mut_ptr());
            gl::DeleteVertexArrays(1, &mut vao);
        }
    }
}
