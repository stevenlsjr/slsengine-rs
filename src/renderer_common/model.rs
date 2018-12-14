// Gltf Model
// and scene presentation structure
use math::*;

use cgmath::*;
use gltf;
use gltf::mesh;
use renderer::Mesh;
use std::{collections::HashMap, path::Path};

use failure;

#[derive(Clone, PartialEq, Debug)]
pub struct MeshData {
    pub mesh: Mesh,
    pub draw_mode: mesh::Mode,
    pub transform_index: Option<usize>,
    pub material_index: Option<usize>,
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

struct GltfImport {
    document: gltf::Document,
    buffers: Vec<gltf::buffer::Data>,
    images: Vec<gltf::image::Data>,
}

pub struct Model {
    pub meshes: Vec<MeshData>,
    pub transforms: Vec<Mat4>,
    pub materials: HashMap<Option<usize>, super::material::UntexturedMat>,
    imports: GltfImport,
}

impl Model {
    fn new(imports: GltfImport) -> Self {
        Model {
            meshes: Vec::new(),
            transforms: Vec::new(),
            materials: HashMap::new(),
            imports,
        }
    }

    fn load_materials(&mut self) {
        for material in self.imports.document.materials() {
            use super::material::*;
            let pbr = material.pbr_metallic_roughness();
            println!("found material {:?}", material.name());
            let game_mat = Material::<Untextured>::new(
                pbr.base_color_factor().into(),
                pbr.metallic_factor(),
                pbr.roughness_factor(),
                material.emissive_factor().into(),
            );
            self.materials.insert(material.index(), game_mat);
        }
    }

    pub fn from_gltf<P: AsRef<Path>>(path: P) -> Result<Self, failure::Error> {
        let (document, buffers, images) = gltf::import(path)?;
        let mut model = Model::new(GltfImport {
            document,
            buffers,
            images,
        });
        {
            let GltfImport {
                document, buffers, ..
            } = &model.imports;

            model.transforms.push(Mat4::identity());
            for ref g_mesh in document.meshes() {
                let meshes: Vec<_> = make_mesh(g_mesh, &buffers)?;
                for m in meshes {
                    let mut md = MeshData::new(m.mesh, m.mode);
                    md.material_index = m.material;

                    model.meshes.push(md);
                }
            }
        }

        model.load_materials();
        Ok(model)
    }
}

struct ParsedMesh {
    mesh: Mesh,
    mode: mesh::Mode,
    material: Option<usize>,
}

impl ParsedMesh {}

fn make_mesh(
    gltf_mesh: &gltf::Mesh,
    buffers: &[gltf::buffer::Data],
) -> Result<Vec<ParsedMesh>, failure::Error> {
    use renderer::Vertex as SlsVertex;
    let mut meshes = Vec::new();
    let get_buffer_data =
        |buffer: gltf::Buffer| Some(&*buffers[buffer.index()]);

    for primitive in gltf_mesh.primitives() {
        let reader = primitive.reader(get_buffer_data);
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
            mesh: Mesh { indices, vertices },
            mode: primitive.mode(),
            material: primitive.material().index(),
        };
        meshes.push(mesh_data);
    }
    Ok(meshes)
}

#[cfg(test)]
mod test {
    use super::*;

    static GLB_BYTES: &[u8] = include_bytes!("../../assets/models/earth.glb");
    #[test]
    fn test_model_loader() {
        use gltf::{binary::Glb, Gltf};
        let glb = Glb::from_slice(GLB_BYTES)
            .expect("file should be valid gltf binary bytecode");
        let gl_model = Gltf::from_slice(GLB_BYTES)
            .expect("should load valid gltf document");
        let model_res = Model::from_gltf(&gl_model);
        if let Ok(model) = model_res {
            assert_eq!(
                model.meshes.len(),
                1,
                "Model should load mesh with a single model"
            );
            let mesh = model.meshes[0].clone();
            assert!(model.materials.contains_key(&mesh.material_index));
        } else {
            panic!("No meshes found")
        }
    }
}
