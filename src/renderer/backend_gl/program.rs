use super::errors::*;
use cgmath::*;
use gl;
use gl::types::GLenum;
use log::*;
use std::borrow::Borrow;
use std::cell::{BorrowMutError, Ref, RefCell};
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::Path;

pub struct ShaderStage(pub u32);

#[derive(Debug)]
pub struct Program<Uniforms>
where
    Uniforms: ShaderUniforms,
{
    pub id: u32,
    pub uniforms: RefCell<Uniforms>,
}

pub type PbrProgram = Program<PbrShaderUniforms>;

pub struct ProgramBuilder {
    frag_shader: Option<u32>,
    vert_shader: Option<u32>,
    // geometry_shader: Option<u32>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum TextureUnits {
    Albedo = 0,
    MetallicRoughness = 1,
    Normal = 2,
    Ao = 3,
    Emissive = 4,
}

impl TextureUnits {
    /// returns the opengl texture unit for given
    /// Texture map
    fn gl_unit(self) -> GLenum {
        gl::TEXTURE0 + self as u32
    }
}

pub trait ShaderUniforms: Debug + Sized {
    fn find_locations(&mut self, program: &Program<Self>);
}

//
/// Stores uniform ids for the main
/// scene shader
#[derive(Clone, Debug)]
pub struct PbrShaderUniforms {
    pub modelview: Option<u32>,
    pub projection: Option<u32>,
    pub normal_matrix: Option<u32>,
    pub light_positions: Option<u32>,
    pub albedo_map: Option<u32>,
    pub metallic_roughness_map: Option<u32>,
    pub normal_map: Option<u32>,
    pub ao_map: Option<u32>,
    pub emissive_map: Option<u32>,
    user_uniforms: HashMap<String, Option<u32>>,
}

impl PbrShaderUniforms {
    fn set_texture_units(&self, program: &Program<Self>) {
        // set texture units
        let units = [
            (self.albedo_map, TextureUnits::Albedo),
            (self.metallic_roughness_map, TextureUnits::MetallicRoughness),
            (self.normal_map, TextureUnits::Normal),
            (self.ao_map, TextureUnits::Ao),
            (self.emissive_map, TextureUnits::Emissive),
        ];
        program.use_program();
        for (location_opt, unit) in &units {
            if let Some(location) = *location_opt {
                unsafe {
                    gl::Uniform1i(location as _, *unit as i32);
                }
            }
        }
    }
}

impl ShaderUniforms for PbrShaderUniforms {
    fn find_locations(&mut self, program: &Program<Self>) {
        {
            let uniforms = &mut [
                (&mut self.modelview, "modelview"),
                (&mut self.projection, "projection"),
                (&mut self.normal_matrix, "normal_matrix"),
                (&mut self.light_positions, "light_positions"),
                (&mut self.albedo_map, "albedo_map"),
                (&mut self.metallic_roughness_map, "metallic_roughness_map"),
                (&mut self.normal_map, "normal_map"),
                (&mut self.ao_map, "ao_map"),
                (&mut self.emissive_map, "emissive_map"),
            ];
            for (ref mut ptr, name) in uniforms {
                **ptr = program.uniform_location(name).unwrap_or_else(|_e| {
                    warn!("could not bind location '{}'", name);
                    None
                });
            }
        }

        for (key, value) in self.user_uniforms.iter_mut() {
            *value = program.uniform_location(key).unwrap_or_else(|_e| {
                warn!("could not bind location '{}'", key);
                None
            });
        }

        self.set_texture_units(program);
    }
}

impl Default for PbrShaderUniforms {
    fn default() -> Self {
        PbrShaderUniforms {
            modelview: None,
            projection: None,
            normal_matrix: None,
            light_positions: None,
            albedo_map: None,
            metallic_roughness_map: None,
            normal_map: None,
            ao_map: None,
            emissive_map: None,
            user_uniforms: HashMap::new(),
        }
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

impl<U> Program<U>
where
    U: ShaderUniforms,
{
    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    #[inline]
    pub fn uniforms(&self) -> Ref<U> {
        self.uniforms.borrow()
    }

    pub fn try_set_uniforms(&self) -> Result<(), BorrowMutError> {
        let mut u = self.uniforms.try_borrow_mut()?;
        u.find_locations(self);
        Ok(())
    }

    /// Binds textures in pbr material to shader samplers
    pub fn bind_material_textures(
        &self,
        material: &super::ManagedTextureMaterial,
    ) {
        self.use_program();
        // self.uniforms.borrow().set_texture_units(self);
        let units = [
            (&material.albedo_map, TextureUnits::Albedo),
            (
                &material.metallic_roughness_map,
                TextureUnits::MetallicRoughness,
            ),
            (&material.normal_map, TextureUnits::Normal),
            (&material.occlusion_map, TextureUnits::Ao),
            (&material.emissive_map, TextureUnits::Emissive),
        ];

        {
            let _m = material.borrow();
        }

        for (texture_opt, unit) in &units {
            if let Some(texture) = texture_opt {
                unsafe {
                    gl::ActiveTexture(unit.gl_unit());
                    gl::BindTexture(gl::TEXTURE_2D, texture.id());
                }
            }
        }
    }

    pub fn uniform_location(
        &self,
        name: &str,
    ) -> Result<Option<u32>, ShaderError> {
        use std::ffi::CString;
        let cs_name = CString::new(name).map_err(|e| {
            ShaderError::UniformBindFailure {
                name: name.to_string(),
                msg: format!("name can't be converted to c string: {}", e),
            }
        })?;
        let id = unsafe { gl::GetUniformLocation(self.id, cs_name.as_ptr()) };
        if id < 0 {
            return Ok(None);
        }
        Ok(Some(id as _))
    }
}

impl<U> Drop for Program<U>
where
    U: ShaderUniforms,
{
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.id) }
    }
}

pub trait BindUniform<T> {
    type Id;
    fn bind_uniform(&self, id: Self::Id, val: &T);
}

impl<U> BindUniform<Matrix4<f32>> for Program<U>
where
    U: ShaderUniforms,
{
    type Id = Option<u32>;
    fn bind_uniform(&self, id: Self::Id, val: &Matrix4<f32>) {
        if let Some(id) = id {
            unsafe {
                gl::UniformMatrix4fv(id as _, 1, gl::FALSE, val.as_ptr());
            }
        }
    }
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

    pub fn build_program<U>(
        &self,
        uniforms: U,
    ) -> Result<Program<U>, ShaderError>
    where
        U: ShaderUniforms,
    {
        let id = unsafe { self.link_program() }?;
        let prog = Program {
            id,
            uniforms: RefCell::new(uniforms),
        };
        prog.uniforms.borrow_mut().find_locations(&prog);

        Ok(prog)
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

pub fn program_from_sources<P, U>(
    vs_path: P,
    fs_path: P,
    uniforms: U,
) -> Result<Program<U>, ShaderError>
where
    P: AsRef<Path> + Debug,
    U: ShaderUniforms,
{
    let header: &'static str = "#version 410\n";
    use std::fs;
    let vs_source = fs::read_to_string(&vs_path).map_err(|e| {
        ShaderError::CompileFailure {
            info_log: format!("Error opening frag shader source {}", e),
        }
    })?;

    let fs_source = fs::read_to_string(&fs_path).map_err(|e| {
        ShaderError::CompileFailure {
            info_log: format!("Error opening vertex shader source {}", e),
        }
    })?;

    let vs =
        unsafe { compile_source(&[header, &vs_source], gl::VERTEX_SHADER) }?;
    let fs =
        unsafe { compile_source(&[header, &fs_source], gl::FRAGMENT_SHADER) }?;

    ProgramBuilder::new()
        .frag_shader(fs.0)
        .vert_shader(vs.0)
        .build_program(uniforms)
}
