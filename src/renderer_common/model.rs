// Gltf Model
// and scene presentation structure
use math::*;

use cgmath::*;
use gltf;
use gltf::{mesh};
use renderer::Mesh;

use failure;
pub struct PlaceholdMaterial;
pub type Material = PlaceholdMaterial;

#[derive(Clone, PartialEq, Debug)]
pub struct MeshData {
    mesh: Mesh,
    draw_mode: mesh::Mode,
    transform_index: Option<usize>,
    material_index: Option<usize>,
}
impl MeshData {
    fn new(mesh: Mesh, draw_mode: mesh::Mode) -> Self {
        MeshData {
            mesh,
            draw_mode,
            transform_index: None,
            material_index: None,
        }
    }
}

pub struct Model {
    pub meshes: Vec<MeshData>,
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
        let blob = &gltf_file
            .blob
            .clone()
            .ok_or(format_err!("loader only supports glb files"))?;
        model.transforms.push(Mat4::identity());
        for ref g_mesh in gltf_file.document.meshes() {
            let meshes: Vec<_> = make_mesh(g_mesh, blob)?;
            for m in meshes {
                let md = MeshData::new(m.mesh, m.mode);
                model.meshes.push(md);
            }
        }
        Ok(model)
    }
}

struct ParsedMesh<'a> {
    mesh: Mesh,
    mode: mesh::Mode,
    material: gltf::Material<'a>,
}

fn make_mesh<'a, 'b>(
    gltf_mesh: &'b gltf::Mesh<'a>,
    blob: &[u8],
) -> Result<Vec<ParsedMesh<'a>>, failure::Error> {
    use renderer::Vertex as SlsVertex;
    let mut meshes = Vec::new();

    for primitive in gltf_mesh.primitives() {
        let reader = primitive.reader(|_buffer| Some(blob));
        let positions: Vec<_> = reader
            .read_positions()
            .map(|iter| iter.collect())
            .ok_or(format_err!("mesh primitive is missing positions"))?;

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
        let indices: Vec<u32> = if let Some(index_enum) = reader.read_indices()
        {
            index_enum.into_u32().collect()
        } else {
            panic!("model doesn't have indices");
        };
        let mesh_data = ParsedMesh {
            mesh: Mesh{indices, vertices},
            mode: primitive.mode(),
            material: primitive.material(),
        };
        meshes.push(mesh_data);
    }
    Ok(meshes)
}

#[cfg(test)]
mod test {
    use super::*;

    static GLB_BYTES: &[u8] = include_bytes!("../../assets/earth.glb");
    #[test]
    fn test_model_loader() {
        use gltf::{binary::Glb, Gltf};
        let glb = Glb::from_slice(GLB_BYTES)
            .expect("file should be valid gltf binary bytecode");
        let gl_model = Gltf::from_slice(GLB_BYTES)
            .expect("should load valid gltf document");
        let model_res = Model::from_gltf(&gl_model);
        assert!(model_res.is_ok());
        if let Ok(model) = model_res {
            assert_eq!(
                model.meshes.len(),
                1,
                "Model should load mesh with a single model"
            );
        }
    }
}
