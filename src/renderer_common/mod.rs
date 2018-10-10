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
