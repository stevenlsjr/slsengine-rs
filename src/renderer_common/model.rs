// Gltf Model
// and scene presentation structure
use math::*;

use cgmath::*;
use gltf;
use renderer::Mesh;

use failure;
pub struct PlaceholdMaterial;
pub type Material = PlaceholdMaterial;

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub transforms: Vec<Mat4>,
    pub materials: Vec<Material>,
}

impl Model {
    fn new() -> Self {
        Model {
            meshes: Vec::new(),
            transforms: Vec::new(),
            materials: Vec::new(),
        }
    }
    pub fn from_gltf(gltf_file: &gltf::Gltf) -> Result<Self, failure::Error> {
        let mut model = Model::new();
        let blob = &gltf_file.blob.clone().ok_or(format_err!("loader only supports glb files"))?;
        model.transforms.push(Mat4::identity());
        for ref g_mesh in gltf_file.document.meshes() {
            let mesh = make_mesh(g_mesh, blob)?;
            model.meshes.push(mesh);

        }
        Ok(model)
    }
}

fn make_mesh(
    gltf_mesh: &gltf::Mesh,
    blob: &[u8],
) -> Result<Mesh, failure::Error> {
    use renderer::Vertex as SlsVertex;

    let primitive = gltf_mesh.primitives().next().unwrap();
    let reader = primitive.reader(|_buffer| Some(blob));
    let positions: Vec<_> = reader
        .read_positions()
        .map(|iter| iter.collect()).ok_or(format_err!("mesh primitive is missing positions"))?;
        
    let mut vertices: Vec<SlsVertex> = positions
        .iter()
        .map(|pos| SlsVertex {
            position: pos.clone(),
            ..SlsVertex::default()
        })
        .collect();

    if let Some(normals) = reader.read_normals() {
        for (i, normal) in normals.enumerate() {
            vertices[i].normal = normal.clone();
        }
    }
    if let Some(uvs) = reader.read_tex_coords(0) {
        for (i, uv) in uvs.into_f32().enumerate() {
            vertices[i].uv = uv.clone();
        }
    }
    let indices: Vec<u32> = if let Some(index_enum) = reader.read_indices() {
        index_enum.into_u32().collect()
    } else {
        panic!("model doesn't have indices");
    };
    Ok(Mesh { vertices, indices })
}
