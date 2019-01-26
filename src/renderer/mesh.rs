use super::color::ColorRGBA;
use cgmath::*;
use genmesh::{
    generators::{IndexedPolygon, SharedVertex},
    Triangle,
};
use std::slice::Chunks;

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
    fn mesh(&self) -> &Mesh;
    fn vertices(&self) -> &[Vertex] {
        &self.mesh().vertices
    }
    fn indices(&self) -> &[u32] {
        &self.mesh().indices
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

    pub fn triangle_indices(&self) -> Chunks<u32> {
        self.indices.chunks(3)
    }

    /// takes a mutable mesh, and sets the vertex tangents and bitangents
    pub fn calculate_tangents(&mut self) {
        let mut tangents = vec![Vec::new(); self.vertices.len()];
        let mut bitangents = vec![Vec::new(); self.vertices.len()];

        for tri in self.indices.chunks(3) {
            if tri.len() < 3 {
                break;
            }
            let v0 = self.vertices[tri[0] as usize];
            let v1 = self.vertices[tri[1] as usize];
            let v2 = self.vertices[tri[2] as usize];
            let delta_pos1 =
                Vector3::from(v1.position) - Vector3::from(v0.position);
            let delta_pos2 =
                Vector3::from(v2.position) - Vector3::from(v0.position);
            let delta_uv1 = Vector2::from(v1.uv) - Vector2::from(v0.uv);
            let delta_uv2 = Vector2::from(v2.uv) - Vector2::from(v0.uv);
            let f =
                1.0 / (delta_uv1.x * delta_uv2.y + delta_uv2.x * delta_uv1.y);
            let tangent =
                f * (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y);
            let bitangent =
                f * (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x);
            for &i in tri {
                tangents[i as usize].push(tangent);
                bitangents[i as usize].push(bitangent);
            }
        }

        // take average of
        for (i, v) in self.vertices.iter_mut().enumerate() {
            let t: &[Vector3<f32>] = &tangents[i];
            let b: &[Vector3<f32>] = &bitangents[i];
            if t.len() == 0 || b.len() == 0 {
                continue;
            }
            let t_sum: Vector3<_> = t.iter().sum();
            v.tangent = (t_sum / t.len() as f32).into();
            let b_sum: Vector3<_> = b.iter().sum();
            v.bitangent = (b_sum / b.len() as f32).into();
        }
    }

    /// Creates a mesh from a genmesh geometry.
    pub fn from_genmesh<G>(generator: G) -> Self
    where
        G: SharedVertex<genmesh::Vertex> + IndexedPolygon<Triangle<usize>>,
    {
        let mut m = Mesh {
            vertices: generator
                .shared_vertex_iter()
                .map(|v| Vertex {
                    position: v.pos.into(),
                    normal: v.normal.into(),
                    ..Vertex::default()
                })
                .collect(),
            indices: generator
                .indexed_polygon_iter()
                .flat_map(|t| vec![t.x as u32, t.y as u32, t.z as u32])
                .collect(),
        };
        m.calculate_tangents();
        m
    }
}

impl RenderMesh for Mesh {
    fn mesh(&self) -> &Mesh {
        self
    }
}

impl SharedVertex<Vertex> for Mesh {
    fn shared_vertex(&self, i: usize) -> Vertex {
        self.vertices[self.indices[i] as usize]
    }
    fn shared_vertex_count(&self) -> usize {
        self.indices.len()
    }
}
