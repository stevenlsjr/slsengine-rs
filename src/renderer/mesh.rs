use cgmath::*;

/// A cffi and GPU-friendly vertex representaion
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}



impl Default for Vertex {
    fn default() -> Self {
        Vertex {
            position: [0., 0., 0.],
            normal: [0., 0., 1.],
            tangent: [0., 0., 0.],
            bitangent: [0., 0., 0.],
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

pub trait RenderMesh {
    fn vertices(&self)->&[Vertex];
    fn indices(&self)->&[u32];
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

impl RenderMesh for Mesh {
    fn vertices(&self)->&[Vertex] {&self.vertices}
    fn indices(&self)->&[u32] {&self.indices}
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
