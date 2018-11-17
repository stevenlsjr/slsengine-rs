use cgmath::*;
use sdl2::video::Window;
use std::{cell, time::Duration};

/// A cffi and GPU-friendly vertex representaion
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

impl Default for Vertex {
    fn default() -> Self {
        Vertex {
            position: [0., 0., 0.],
            normal: [0., 0., 1.],
            uv: [0., 0.],
            color: [1., 1., 1., 1.],
        }
    }
}

impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
            && self.normal == other.normal
            && self.uv == other.uv
            && self.color == other.color
    }

    fn ne(&self, other: &Self) -> bool {
        !PartialEq::eq(self, other)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn new() -> Mesh {
        Mesh {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn verts_size(&self) -> usize {
        use std::mem::size_of;
        (self.vertices.len() * size_of::<Vertex>())
    }
    pub fn indices_size(&self) -> usize {
        use std::mem::size_of;
        (self.indices.len() * size_of::<u32>())
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

#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

pub fn color4f(r: f32, g: f32, b: f32, a: f32) -> Color {
    Color { r, g, b, a }
}

impl Into<Vector4<f32>> for Color {
    fn into(self) -> Vector4<f32> {
        ::cgmath::vec4(self.r, self.g, self.b, self.a)
    }
}

impl From<Vector4<f32>> for Color {
    fn from(v: Vector4<f32>) -> Self {
        Color {
            r: v.x,
            g: v.y,
            b: v.z,
            a: v.w,
        }
    }
}

pub trait ShaderProgram<T: Renderer> {
    fn use_program(&self, renderer: &T);
}

pub trait Renderer {
    fn clear(&self) {}
    fn camera(&self) -> cell::Ref<Camera>;
    fn set_clear_color(&mut self, color: Color) {}
    fn on_resize(&self, _size: (u32, u32)) {}
}

pub trait Resizable {
    fn on_resize(&mut self, size: (u32, u32));
}

/*
 *  Camera
 **/

pub fn default_perspective() -> PerspectiveFov<f32> {
    PerspectiveFov {
        fovy: Deg(45.0).into(),
        aspect: 1.0,
        near: 0.1,
        far: 1000.0,
    }
}

pub struct Camera {
    pub projection: Matrix4<f32>,
    perspective: PerspectiveFov<f32>,
}

use std::fmt;
impl fmt::Debug for Camera {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Camera")
    }
}

impl Camera {
    pub fn new(perspective: PerspectiveFov<f32>) -> Camera {
        let cam = Camera {
            perspective,
            projection: perspective.into(),
        };
        cam
    }

    pub fn perspective(&self) -> PerspectiveFov<f32> {
        self.perspective
    }

    fn build_perspective(&mut self) {
        self.projection = self.perspective.into();
    }
}

impl Resizable for Camera {
    fn on_resize(&mut self, (width, height): (u32, u32)) {
        let aspect = width as f32 / height as f32;
        self.perspective.aspect = aspect;
        self.build_perspective();
    }
}
