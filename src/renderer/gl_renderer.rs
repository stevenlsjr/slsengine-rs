use cgmath::prelude::*;
use cgmath::*;
use core;
use failure;
use gl;
pub use renderer_common::*;
use sdl2::video::Window;
use std::cell::{Cell, Ref, RefCell};
use std::time::Instant;

#[derive(Fail, Debug)]
pub enum RendererError {
    #[fail(display = "Renderer lifecycle failed: {}", reason)]
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

#[derive(Debug, PartialEq)]
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

fn create_scene_shaders() -> Result<Program, ShaderError> {
    use renderer::ShaderError;
    use renderer::*;
    use std::fs::File;
    use std::io::Read;
    let header: &'static str = "#version 410\n";
    let mut vs_source = String::new();
    let mut fs_source = String::new();

    {
        let mut vsf = File::open("./assets/blinn-phong.vert").map_err(|e| {
            ShaderError::CompileFailure {
                info_log: format!("Error opening source {}", e),
            }
        })?;
        let mut fsf = File::open("./assets/blinn-phong.frag").map_err(|e| {
            ShaderError::CompileFailure {
                info_log: format!("Error opening source {}", e),
            }
        })?;

        vsf.read_to_string(&mut vs_source).map_err(|_| {
            ShaderError::CompileFailure {
                info_log: "could not read vert shader".to_string(),
            }
        })?;
        fsf.read_to_string(&mut fs_source).map_err(|_| {
            ShaderError::CompileFailure {
                info_log: "could not read vert shader".to_string(),
            }
        })?;
    }

    let vs =
        unsafe { compile_source(&[header, &vs_source], gl::VERTEX_SHADER) }?;

    let fs =
        unsafe { compile_source(&[header, &fs_source], gl::FRAGMENT_SHADER) }?;

    ProgramBuilder::new()
        .frag_shader(fs.0)
        .vert_shader(vs.0)
        .build_program()
}

/*
 * Opengl renderer implementation
 */

///
/// Stores uniform ids for the main
/// scene shader
#[derive(Clone, Debug)]
pub struct SceneUniforms {
    pub modelview: i32,
    pub projection: i32,
    pub normal_matrix: i32,
    pub light_positions: i32,
}
impl Default for SceneUniforms {
    fn default() -> Self {
        SceneUniforms {
            modelview: -1,
            projection: -1,
            normal_matrix: -1,
            light_positions: -1,
        }
    }
}

struct Materials {
    base_material: material::UntexturedMat,
    base_material_ubo: super::objects::MaterialUbo,
}

/// the renderer backend for openGL
pub struct GlRenderer {
    camera: RefCell<Camera>,
    scene_program: Program,
    scene_uniforms: SceneUniforms,
    sample_mesh: Mesh,
    buffers: super::objects::MeshBuffers,
    materials: Materials,

    recompile_flag: Cell<Option<Instant>>,
}

impl GlRenderer {
    pub fn projection(&self) -> Matrix4<f32> {
        self.camera.borrow().projection
    }

    fn initialize(&mut self) -> Result<(), failure::Error> {
        self.bind_uniforms();
        self.materials.base_material_ubo.bind_to_material(
            &self.scene_program,
            &self.materials.base_material,
        );

        Ok(())
    }

    pub fn new(
        window: &Window,
        mesh: Mesh,
    ) -> Result<GlRenderer, RendererError> {
        use super::objects::*;
        let (width, height) = window.size();
        let perspective = PerspectiveFov {
            fovy: Deg(40.0).into(),
            aspect: width as f32 / height as f32,
            near: 0.1,
            far: 1000.0,
        };
        let scene_program =
            create_scene_shaders().map_err(RendererError::ShaderError)?;
        let scene_uniforms = SceneUniforms::default();

        let buffers =
            MeshBuffers::new().map_err(|_| RendererError::Lifecycle {
                reason: format!("could not build gl objects for mesh"),
            })?;
        buffers
            .bind_mesh(&mesh)
            .map_err(|_| RendererError::Lifecycle {
                reason: format!("could not bind buffers to mesh"),
            })?;

        let materials = GlRenderer::get_materials().map_err(|_| {
            RendererError::Lifecycle {
                reason: format!("could create material_obo"),
            }
        })?;

        let mut renderer = GlRenderer {
            scene_program,
            scene_uniforms,
            sample_mesh: mesh,
            buffers,
            materials,
            camera: RefCell::new(Camera::new(perspective)),
            recompile_flag: Cell::new(None),
        };
        if let Err(e) = renderer.initialize() {
            return Err(RendererError::Lifecycle {
                reason: format!("could not initialize renderer: {:?}", e),
            });
        }

        renderer.on_resize((width, height));

        Ok(renderer)
    }

    fn get_materials() -> Result<Materials, ::failure::Error> {
        use super::objects::*;
        use failure::Error;
        let base_material: material::UntexturedMat =
            material::UntexturedMat::new(
                vec4(1.0, 1.0, 0.0, 1.0),
                1.0,
                1.0,
                vec3(0.0, 0.0, 0.0),
            );
        let base_material_ubo = MaterialUbo::new().map_err(&Error::from)?;
        Ok(Materials {
            base_material,
            base_material_ubo,
        })
    }

    fn bind_uniforms(&mut self) {
        self.scene_uniforms.modelview = self
            .scene_program
            .uniform_location("modelview")
            .unwrap_or(-1);
        self.scene_uniforms.projection = self
            .scene_program
            .uniform_location("projection")
            .unwrap_or(-1);
        self.scene_uniforms.normal_matrix = self
            .scene_program
            .uniform_location("normal_matrix")
            .unwrap_or(-1);

        self.scene_uniforms.light_positions = self
            .scene_program
            .uniform_location("light_positions")
            .unwrap_or(-1);
    }

    pub fn rebuild_program(&mut self) {
        println!(
            "old shader program {:#?} with uniforms {:#?}",
            self.scene_program, self.scene_uniforms
        );

        let program = match create_scene_shaders() {
            Ok(mut program) => program,
            Err(e) => {
                eprintln!("could not rebuild shaders: {}", e);
                return;
            }
        };

        self.scene_program = program;
        self.scene_program.use_program();

        self.bind_uniforms();
        self.scene_program.bind_uniform(
            self.scene_uniforms.projection,
            &self.camera.borrow().projection,
        );
        println!(
            "build new shader program {:#?} with uniforms {:#?}",
            self.scene_program, self.scene_uniforms
        );
    }

    #[inline]
    pub fn scene_program(&self) -> &Program {
        &self.scene_program
    }

    #[inline]
    pub fn scene_uniforms(&self) -> &SceneUniforms {
        &self.scene_uniforms
    }
}

impl Renderer for GlRenderer {
    fn clear(&self) {
        unsafe {
            gl::ClearColor(0.6, 0.0, 0.8, 1.0);
            gl::Enable(gl::DEPTH_TEST);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    fn camera(&self) -> Ref<Camera> {
        self.camera.borrow()
    }

    fn set_clear_color(&mut self, color: Color) {
        unsafe {
            gl::ClearColor(color.r, color.g, color.b, color.a);
        }
    }

    fn on_resize(&self, size: (u32, u32)) {
        self.camera.borrow_mut().on_resize(size);
        let (width, height) = size;
        self.scene_program.use_program();
        self.scene_program.bind_uniform(
            self.scene_uniforms.projection,
            &self.camera.borrow().projection,
        );

        unsafe {
            gl::Viewport(0, 0, width as i32, height as i32);
        }
    }

    fn on_update(
        &mut self,
        _delta_time: ::std::time::Duration,
        _world: &game::EntityWorld,
    ) {
        if let Some(t) = self.recompile_flag.get() {
            self.rebuild_program();
            self.recompile_flag.set(None);
        }
    }

    fn flag_shader_recompile(&self) {
        if self.recompile_flag.get().is_none() {
            self.recompile_flag.set(Some(Instant::now()))
        }
    }
}

pub trait RenderScene<S> {
    fn render_scene(&self, mesh: &S);
}

use super::super::game;
impl RenderScene<game::EntityWorld> for GlRenderer {
    fn render_scene(&self, scene: &game::EntityWorld) {
        use std::ptr;

        use math::*;
        let program = self.scene_program();
        let cam_view = scene.main_camera.transform();
        let light_positions: &[Vec3] = &[vec3(0.0, 1.0, -1.0)];
        let xformed_light_positions: Vec<Vec3> = light_positions
            .iter()
            .map(|v| (cam_view * v.extend(1.0)).xyz())
            .collect();
        let SceneUniforms {
            modelview: modelview_id,
            normal_matrix: normal_matrix_id,
            light_positions: light_positions_id,
            ..
        } = self.scene_uniforms().clone();

        let buffers = &self.buffers;
        let mesh = &self.sample_mesh;
        program.use_program();
        unsafe {
            let light_pos_ptr = xformed_light_positions.as_ptr();
            gl::Uniform4fv(light_positions_id, 1, light_pos_ptr as *const _);
        }

        for pos in &scene.sphere_positions {
            let modelview = cam_view * Mat4::from_translation(pos.to_vec());
            let normal_matrix = modelview.invert().unwrap().transpose();
            program.bind_uniform(modelview_id, &modelview);
            program.bind_uniform(normal_matrix_id, &normal_matrix);
            unsafe {
                gl::BindVertexArray(buffers.vertex_array.id());
                gl::DrawElements(
                    gl::TRIANGLES,
                    mesh.indices.len() as i32,
                    gl::UNSIGNED_INT,
                    ptr::null(),
                );
            }
        }
    }
}
