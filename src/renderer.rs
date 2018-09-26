extern crate core;
extern crate failure;
extern crate gl;

use std::ffi::{CStr, CString};

#[derive(Fail, Debug)]
pub enum RendererError {
    #[fail(display = "Failed to construct Geometry: {}", reason)]
    GeometryFailure { reason: String },
}

#[derive(Fail, Debug)]
pub enum ShaderError {
    #[fail(display = "invalid shader type: 0x{:X}", shader_type)]
    InvalidType { shader_type: u32 },

    #[fail(display = "Shader compilation failed, log: `{}`", info_log)]
    CompileFailure { info_log: String },
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    fn new() -> Mesh {
        Mesh {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }
}

pub struct MeshBuilder {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub colors: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
}

impl MeshBuilder {
    pub fn new() -> MeshBuilder {
        MeshBuilder {
            positions: vec![],
            normals: vec![],
            uvs: vec![],
            colors: vec![],
            indices: vec![],
        }
    }
    pub fn build(&self) -> Result<Mesh, String> {
        let len = self.positions.len();

        let normals_len = self.normals.len();
        let uvs_len = self.uvs.len();
        let colors_len = self.colors.len();

        let mut verts: Vec<Vertex> = Vec::new();
        verts.reserve(len);
        for i in 0..len {
            let vertex = Vertex {
                position: self.positions[i].clone(),
                normal: if i < normals_len {
                    self.normals[i].clone()
                } else {
                    [0.0, 0.0, 1.0]
                },
                uv: if i < uvs_len {
                    self.uvs[i].clone()
                } else {
                    [0.0, 0.0]
                },
                color: if i < colors_len {
                    self.colors[i].clone()
                } else {
                    [1.0, 1.0, 1.0, 1.0]
                },
            };
            verts.push(vertex);
        }
        let indices = self.indices.clone();

        Ok(Mesh {
            indices,
            vertices: verts,
        })
    }
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

unsafe fn drain_error_stack() {
    let mut err = gl::NO_ERROR;
    loop {
        err = gl::GetError();
        if err == gl::NO_ERROR {
            break;
        }
    }
}

unsafe fn debug_error_stack() {
    let mut err = gl::NO_ERROR;
    loop {
        err = gl::GetError();
        if err == gl::NO_ERROR {
            break;
        }
        eprintln!("GL error: 0x{:X}", err)
    }
}

pub unsafe fn get_shader_info_log(shader: u32) -> String {
    let mut log_len = 0i32;
    gl::GetShaderInfoLog(shader, 0, &mut log_len, 0 as *mut _);
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
) -> Result<u32, ShaderError> {
    let mut ptr_sources: Vec<*const i8> = Vec::new();
    let mut lengths: Vec<i32> = Vec::new();
    for s in sources {
        lengths.push(s.len() as i32);
        ptr_sources.push(s.as_ptr() as *const i8)
    }

    let shader = gl::CreateShader(shader_type);
    if gl::IsShader(shader) == 0 {
        return Err(ShaderError::CompileFailure {
            info_log: "Could not construct shader".to_string(),
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

    Ok(shader)
}

#[cfg(test)]
mod test {
    extern crate sdl2;

    use super::*;
    use sdl2::video;
    use sdl_platform::{platform, Platform};

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

    fn platform_with_gl() -> (Platform, sdl2::video::GLContext) {
        let plt = platform().build().unwrap();

        let gl_ctx = plt.window.gl_create_context().unwrap();
        gl::load_with(|name| {
            plt.video_subsystem.gl_get_proc_address(name) as *const _
        });

        plt.window.gl_set_context_to_current().unwrap();

        (plt, gl_ctx)
    }

    //    #[test]
    fn test_shader_compile() {
        let (platform, _) = platform_with_gl();
        platform.window.gl_set_context_to_current().unwrap();
        let good_src = r#"
        out vec4 out_color;
        void main(){
            out_color = vec4(1.0, 0.0, 0.0, 1.0);
        }
        "#;

        let bad_src = r#"
        out vec4 out_color;
        void main(){
            out_color = vec4(1.0, 0.0, 0.0, 1.0);

        "#; // unmatched brackets

        let header = "#version 410\n";

        let good_shader =
            unsafe { compile_source(&[header, good_src], gl::FRAGMENT_SHADER) };
        assert!(
            good_shader.is_ok(),
            "shader compilation should be successful: {}",
            good_shader.unwrap_err()
        );
        let bad_shader =
            unsafe { compile_source(&[header, bad_src], gl::FRAGMENT_SHADER) };
        assert!(bad_shader.is_err());
        if let Err(e) = bad_shader {
            assert!(match e {
                ShaderError::CompileFailure { .. } => true,
                _ => false,
            })
        }
    }
}
