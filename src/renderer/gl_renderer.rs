use ::cgmath;
use ::core;
use ::failure;
use ::gl;
use cgmath::*;
use cgmath::prelude::*;
pub use renderer_common::*;
use sdl2;
use sdl2::video::Window;

#[derive(Fail, Debug)]
pub enum RendererError {
    #[fail(display = "Failed to construct Geometry: {}", reason)]
    GeometryFailure { reason: String },
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

pub fn rectangle_mesh() -> MeshBuilder {
    let mut mb = MeshBuilder::new();
    mb.positions.append(&mut vec![
        [-1.0f32, -1.0f32, 0.0f32],
        [1.0, -1.0, 0.0],
        [1.0, 1.0, 0.0],
        [-1.0, 1.0, 0.0],
    ]);

    mb.normals.append(&mut vec![
        [0.0f32, 0.0f32, 1.0f32],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
    ]);

    mb.uvs.append(&mut vec![
        [0.0f32, 0.0f32],
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
    ]);

    mb.indices.append(&mut vec![0, 1, 2, 2, 3, 0]);
    mb
}

pub fn drain_error_stack() {
    loop {
        let err = unsafe { gl::GetError() };
        if err == gl::NO_ERROR {
            break;
        }
    }
}

/// fills up a vector with errors produced by glGetError
/// . dump_errors returns length of errors
pub fn dump_errors(errors: &mut Vec<gl::types::GLenum>) -> usize {
    errors.clear();
    loop {
        let err = unsafe { gl::GetError() };
        if err == gl::NO_ERROR {
            break;
        }
        errors.push(err);
    }
    errors.len()
}

pub fn debug_error_stack(file: &str, line: u32) {
    loop {
        let err = unsafe { gl::GetError() };
        if err == gl::NO_ERROR {
            break;
        }
        eprintln!("{}, {}, GL error: 0x{:X}", file, line, err)
    }
}

pub unsafe fn get_shader_info_log(shader: u32) -> String {
    let mut log_len = 0i32;
    gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut log_len);
    let mut buffer = vec![0u8; log_len as usize];
    gl::GetShaderInfoLog(
        shader,
        log_len,
        core::ptr::null_mut() as *mut _,
        buffer.as_mut_ptr() as *mut i8,
    );

    String::from_utf8(buffer).unwrap_or("Could not read log".to_string())
}

pub unsafe fn compile_source(
    sources: &[&str],
    shader_type: gl::types::GLenum,
) -> Result<ShaderStage, ShaderError> {
    let mut ptr_sources: Vec<*const i8> = Vec::new();
    let mut lengths: Vec<i32> = Vec::new();
    for s in sources {
        lengths.push(s.len() as i32);
        ptr_sources.push(s.as_ptr() as *const i8)
    }

    let shader = gl::CreateShader(shader_type);
    if gl::IsShader(shader) == 0 {
        return Err(ShaderError::ApiFailure {
            reason: "Could not construct shader".to_string(),
        });
    }

    gl::ShaderSource(
        shader,
        sources.len() as i32,
        ptr_sources.as_ptr(),
        lengths.as_ptr(),
    );
    gl::CompileShader(shader);
    let mut compile_status: i32 = 1;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut compile_status);
    let cs_1 = compile_status;
    compile_status = 0;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut compile_status);
    assert_eq!(cs_1, compile_status);
    if compile_status != gl::TRUE as i32 {
        let info_log = get_shader_info_log(shader);
        return Err(ShaderError::CompileFailure { info_log });
    }

    Ok(ShaderStage(shader))
}

pub struct ShaderStage(pub u32);

impl Drop for ShaderStage {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.0);
        }
    }
}

pub fn get_program_info_log(program: u32) -> String {
    let mut log_len = 0;
    unsafe {
        gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut log_len);
    }
    let mut buffer = vec![0u8; (log_len) as usize];
    unsafe {
        gl::GetProgramInfoLog(
            program,
            log_len,
            0 as *mut _,
            buffer.as_mut_ptr() as *mut i8,
        );
    }
    String::from_utf8_lossy(&buffer).to_string()
}

pub struct Program {
    pub id: u32,
}

impl Program {
    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    pub fn uniform_location(&self, name: &str) -> Result<i32, ShaderError> {
        use std::ffi::CString;
        let cs_name = CString::new(name).map_err(|e| {
            ShaderError::UniformBindFailure {
                name: name.to_string(),
                msg: format!("name can't be converted to c string: {}", e),
            }
        })?;
        let id = unsafe { gl::GetUniformLocation(self.id, cs_name.as_ptr()) };
        if id < 0 {
            return Err(ShaderError::UniformBindFailure {
                name: name.to_string(),
                msg: "".to_string(),
            });
        }
        Ok(id)
    }
}

pub trait BindUniform<T> {
    type Id;
    fn bind_uniform(&self, id: Self::Id, val: &T);
}

impl BindUniform<Matrix4<f32>> for Program {
    type Id = i32;
    fn bind_uniform(&self, id: i32, val: &Matrix4<f32>) {
        unsafe {
            gl::UniformMatrix4fv(id, 1, gl::FALSE, val.as_ptr());
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.id) }
    }
}

pub struct ProgramBuilder {
    frag_shader: Option<u32>,
    vert_shader: Option<u32>,
    // geometry_shader: Option<u32>,
}

impl ProgramBuilder {
    pub fn new() -> ProgramBuilder {
        ProgramBuilder {
            frag_shader: None,
            vert_shader: None,
            //            geometry_shader: None,
        }
    }
    pub fn frag_shader(&mut self, frag_shader: u32) -> &mut ProgramBuilder {
        self.frag_shader = Some(frag_shader);
        self
    }

    pub fn vert_shader(&mut self, vert_shader: u32) -> &mut ProgramBuilder {
        self.vert_shader = Some(vert_shader);
        self
    }

    pub fn build_program(&self) -> Result<Program, ShaderError> {
        let id = unsafe { self.link_program() }?;
        Ok(Program { id })
    }

    pub unsafe fn link_program(&self) -> Result<u32, ShaderError> {
        let program = gl::CreateProgram();
        let (vs, fs) = self
            .vert_shader
            .and_then(|vs| self.frag_shader.map(|fs| (vs, fs)))
            .ok_or(ShaderError::LinkFailure {
                reason: "program must have attached vertex and frag shader"
                    .to_string(),
            })?;

        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);

        let mut result = 0;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut result);
        if result != gl::TRUE as i32 {
            let log: String = get_program_info_log(program);
            gl::DeleteProgram(program);
            return Err(ShaderError::LinkFailure { reason: log });
        }
        gl::DetachShader(program, vs);
        gl::DetachShader(program, fs);

        Ok(program)
    }
}

#[cfg(test)]
mod test {
    extern crate sdl2;

    use super::*;

    #[test]
    fn test_rec_mesh() {
        let mb = rectangle_mesh();
        for i in [mb.normals.len(), mb.positions.len(), mb.uvs.len()].iter() {
            assert_eq!(*i, 4);
        }
        assert_eq!(mb.indices.len(), 6);
    }

    #[test]
    fn test_build_mesh() {
        {
            let mb = rectangle_mesh();
            let mesh = rectangle_mesh().build().unwrap();
            assert_eq!(mb.positions.len(), 4);
            assert_eq!(
                mesh.vertices.len(),
                4,
                "number of vertices should equal positions in mesh builder"
            );
        }

        {
            let mut mb = MeshBuilder::new();
            mb.positions = vec![[1.0f32, 1.0, 1.0]];
            let mesh = mb.build().unwrap();
            assert_eq!(mesh.vertices.len(), 1);
        }
    }

    #[test]
    fn test_new_mesh() {
        let m = Mesh::new();

        assert_eq!(m.vertices.len(), 0);
        assert_eq!(m.indices.len(), 0);
    }

    //    #[test]
}

pub struct GlRenderer {
    fovy: Rad<f32>,
    projection: Matrix4<f32>,
}

impl GlRenderer {
    pub fn projection(&self) -> &Matrix4<f32> {
        &self.projection
    }
    pub fn new(window: &Window, fovy: Rad<f32>) -> GlRenderer {
        let mut renderer = GlRenderer {
            fovy,
            projection: Matrix4::identity(),
        };

        renderer.on_resize(window, window.size());

        renderer
    }
}

impl Renderer for GlRenderer {
    fn clear(&self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }
    fn set_clear_color(&mut self, color: Color) {
        unsafe {
            gl::ClearColor(color.r, color.g, color.b, color.a);
        }
    }

    fn on_resize(&mut self, _window: &Window, size: (u32, u32)) {
        let (width, height) = size;
        let aspect = width as f32 / height as f32;

        self.projection = (PerspectiveFov {
            aspect,
            fovy: self.fovy,
            near: 0.01,
            far: 1000.0,
        }).into();

        unsafe {
            gl::Viewport(0, 0, width as i32, height as i32);
        }
    }
}
