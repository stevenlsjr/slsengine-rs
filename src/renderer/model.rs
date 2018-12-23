// Gltf Model
// and scene presentation structure
use crate::math::*;

use crate::renderer::material;
use crate::renderer::Mesh;
use cgmath::*;
use failure;
use gltf;
use gltf::mesh;
use log::*;
use std::{cell::RefCell, collections::HashMap, path::Path};

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

#[derive(Clone)]
struct GltfImport {
    document: gltf::Document,
    buffers: Vec<gltf::buffer::Data>,
    images: Vec<gltf::image::Data>,
}

#[derive(Clone)]
pub struct Model {
    pub meshes: Vec<MeshData>,
    pub transforms: Vec<Mat4>,
    pub materials:
        HashMap<Option<usize>, material::Material<gltf::image::Data>>,
    imports: RefCell<GltfImport>,
}

impl Model {
    fn new(imports: GltfImport) -> Self {
        Model {
            meshes: Vec::new(),
            transforms: Vec::new(),
            materials: HashMap::new(),
            imports: RefCell::new(imports),
        }
    }

    fn load_materials(&mut self) {
        let imports = self.imports.borrow();
        for material in imports.document.materials() {
            use super::material::*;
            let pbr = material.pbr_metallic_roughness();
            info!("found material {:?}", material.name());
            let image_from_index = |idx: usize| -> (Option<gltf::image::Data>) {
                imports.images.get(idx).map(|i| i.clone())
            };
            let mut mat = Material {
                albedo_factor: pbr.base_color_factor().into(),
                metallic_factor: pbr.metallic_factor(),
                roughness_factor: pbr.roughness_factor(),
                emissive_factor: material.emissive_factor().into(),
                albedo_map: pbr
                    .base_color_texture()
                    .map(|i| i.texture().source().index())
                    .and_then(image_from_index),
                metallic_roughness_map: pbr
                    .metallic_roughness_texture()
                    .map(|i| i.texture().source().index())
                    .and_then(image_from_index),
                emissive_map: material
                    .emissive_texture()
                    .map(|i| i.texture().source().index())
                    .and_then(image_from_index),
                occlusion_map: material
                    .occlusion_texture()
                    .map(|i| i.texture().source().index())
                    .and_then(image_from_index),
                normal_map: material
                    .normal_texture()
                    .map(|i| i.texture().source().index())
                    .and_then(image_from_index),
            };

            if mat.metallic_roughness_map.is_some() {
                mat.roughness_factor = 1.0;
                mat.metallic_factor = 1.0;
            }

            self.materials.insert(material.index(), mat);
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
                ref document,
                ref buffers,
                ..
            } = *model.imports.borrow();

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
    use crate::renderer::Vertex as SlsVertex;
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

        if let Some(tangents) = reader.read_tangents() {
            for (i, tan) in tangents.enumerate() {
                let t: Vec3 = vec3(tan[0], tan[1], tan[2]);
                let n: Vec3 = vertices[i].normal.into();
                let bitangent = t.cross(n);
                vertices[i].tangent = t.into();
                vertices[i].bitangent = bitangent.into();
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

    #[test]
    fn test_model_loader() {
        let model_res = Model::from_gltf("assets/models/earth.glb");
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
