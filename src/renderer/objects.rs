/**
 *  objects.rs: Managed OpenGL buffer, texture, and vertex objects
 **/
extern crate failure;

use gl;
use gl::types::*;
use image;
use renderer_common::Mesh;
use std::ops::*;

#[derive(Fail, Debug)]
pub enum ObjectError {
    #[fail(display = "Could not create object: {:?}", reason)]
    ObjectCreationFailure { reason: String },

    #[fail(display = "Could not bind mesh data, '{}'", _0)]
    ObjectBindFailure(String),
}

pub enum BufferObjectTarget {
    ArrayBuffer = gl::ARRAY_BUFFER as isize,
    //    AtomicCounterBuffer = gl::ATOMIC_COUNTER_BUFFER as isize,
    //    CopyReadBuffer = gl::COPY_READ_BUFFER as isize,
    //    CopyWriteBuffer = gl::COPY_WRITE_BUFFER as isize,
    //    DispatchIndirectBuffer = gl::DISPATCH_INDIRECT_BUFFER as isize,
    //    DrawIndirectBuffer = gl::DRAW_INDIRECT_BUFFER as isize,
    ElementArrayBuffer = gl::ELEMENT_ARRAY_BUFFER as isize,
    //    PixelPackBuffer = gl::PIXEL_PACK_BUFFER as isize,
    //    PixelUnpackBuffer = gl::PIXEL_UNPACK_BUFFER as isize,
    //    QueryBuffer = gl::QUERY_BUFFER as isize,
    //    ShaderStorageBuffer = gl::SHADER_STORAGE_BUFFER as isize,
    TextureBuffer = gl::TEXTURE_BUFFER as isize,
    //    TransformFeedbackBuffer = gl::TRANSFORM_FEEDBACK_BUFFER as isize,
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
            gl::GenBuffers(2, buffers.as_mut_ptr());

            gl::GenVertexArrays(1, &mut vao);
        }

        Ok(MeshBuffers {
            vertex_buffer: BufferObject(buffers[0]),
            index_buffer: BufferObject(buffers[1]),
            vertex_array: VertexArrayObject(vao),
        })
    }

    pub fn bind_mesh(&self, mesh: &Mesh) -> Result<&Self, failure::Error> {
        use renderer_common::Vertex;
        use std::mem::size_of;
        unsafe {
            gl::BindVertexArray(self.vertex_array.id());
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer.id());
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.index_buffer.id());

            gl::BufferData(
                gl::ARRAY_BUFFER,
                mesh.verts_size() as isize,
                mesh.vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                mesh.indices_size() as isize,
                mesh.indices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                size_of::<Vertex>() as i32,
                offset_of!(Vertex, position) as *const _,
            );
            gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                size_of::<Vertex>() as i32,
                offset_of!(Vertex, normal) as *const _,
            );

            gl::VertexAttribPointer(
                2,
                2,
                gl::FLOAT,
                gl::FALSE,
                size_of::<Vertex>() as i32,
                offset_of!(Vertex, uv) as *const _,
            );

            gl::EnableVertexAttribArray(0);
            gl::EnableVertexAttribArray(1);
            gl::EnableVertexAttribArray(2);
        }

        Ok(&self)
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


#[derive(Debug)]
pub struct TextureObjects {
    ids: Vec<u32>,
}

impl TextureObjects {
    pub fn new(len: usize) -> Result<TextureObjects, ObjectError> {
        let mut ids: Vec<u32> = vec![0; len];

        unsafe {
            gl::GenTextures(len as i32, ids.as_mut_ptr());
        }

        Ok(TextureObjects { ids})
    }

    pub fn ids(&self) -> &[u32] {
        &self.ids
    }

    pub fn len(&self) -> usize {
        self.ids.len()
    }
}

impl Drop for TextureObjects {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(self.ids.len() as i32, self.ids.as_mut_ptr());
        }

        self.ids.clear();
    }
}
