use cgmath::*;
use failure;
use gltf;
use crate::math::*;
use std::fmt;

#[derive(Clone)]
pub struct Material<Tex> {
    pub albedo_factor: Vec4,
    pub albedo_map: Option<Tex>,
    pub metallic_factor: f32,
    pub metallic_roughness_map: Option<Tex>,
    pub roughness_factor: f32,
    pub emissive_factor: Vec3,
    pub emissive_map: Option<Tex>,
    pub normal_map: Option<Tex>,
    pub occlusion_map: Option<Tex>,
}

impl<Tex> fmt::Debug for Material<Tex>{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let map: &[(&str, &fmt::Debug)] = &[
            ("albedo_factor", &self.albedo_factor),
            ("metallic_factor", &self.metallic_factor),
            ("roughness_factor", &self.roughness_factor),
            ("emissive_factor", &self.emissive_factor),
        ];
        write!(f, "Material<Tex>")?;
        f.debug_map().entries(map.iter().map(|&(ref k, ref v)| (k, v))).finish()?;
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MaterialMapName {
    Albedo,
    MetallicRoughness,
    Emissive,
    Normal,
    Occlusion
}

impl<Tex> Material<Tex> {
    pub fn new(
        albedo_factor: Vec4,
        metallic_factor: f32,
        roughness_factor: f32,
        emissive_factor: Vec3,
    ) -> Self {
        Material {
            albedo_factor,
            metallic_factor,
            roughness_factor,
            emissive_factor,
            ..Material::default()
        }
    }

    pub fn transform_textures<BTex, F>(&self, f: F) -> Material<BTex>
    where
        F: Fn(&Tex, MaterialMapName) -> Option<BTex>,
    {
        let f = &f;
        let mut mat: Material<BTex> = Material { 
            albedo_factor: self.albedo_factor,
            metallic_factor: self.metallic_factor,
            roughness_factor: self.roughness_factor,
            emissive_factor: self.emissive_factor,
            ..Material::default()
        };
        mat.albedo_map = self.albedo_map.as_ref().and_then(|tex| f(tex, MaterialMapName::Albedo));
        mat.metallic_roughness_map =
            self.metallic_roughness_map.as_ref().and_then(|tex| f(tex, MaterialMapName::MetallicRoughness));
        mat.emissive_map = self.emissive_map.as_ref().and_then(|tex| f(tex, MaterialMapName::Emissive));
        mat.occlusion_map = self.occlusion_map.as_ref().and_then(|tex| f(tex, MaterialMapName::Occlusion));
        mat.normal_map = self.normal_map.as_ref().and_then(|tex| f(tex, MaterialMapName::Normal));
        mat
    }
}

#[test]
fn test_transform_texture() {
    let m = Material {
        albedo_map: Some(1),
        occlusion_map: None,
        ..Material::default()
    };
    let m2 = m.transform_textures(|tex, _| Some(tex * 2));
    assert_eq!(m2.albedo_map, Some(2));
    assert_eq!(m2.occlusion_map, None);
}

impl<Tex> Default for Material<Tex> {
    fn default() -> Self {
        Material {
            albedo_factor: vec4(1.0, 1.0, 1.0, 1.0),
            albedo_map: None,
            metallic_factor: 0.0,
            metallic_roughness_map: None,
            roughness_factor: 1.0,
            emissive_factor: vec3(0.0, 0.0, 0.0),
            emissive_map: None,
            normal_map: None,
            occlusion_map: None,
        }
    }
}

pub fn from_gltf_material(
    gltf_mat: &gltf::Material,
    images: &[gltf::image::Data],
) -> Material<gltf::image::Data> {
    let load_tex = |opt: Option<gltf::texture::Info>| {
        opt.and_then(|info| {
            let tex = info.texture();
            let idx = tex.source().index();
            if idx < images.len() {
                Some(images[idx].clone())
            } else {
                None
            }
        })
    };
    let pbr = gltf_mat.pbr_metallic_roughness();
    let mut mat = Material {
        albedo_factor: pbr.base_color_factor().into(),
        emissive_factor: gltf_mat.emissive_factor().into(),
        metallic_factor: pbr.metallic_factor(),
        roughness_factor: pbr.roughness_factor(),
        ..Material::default()
    };
    mat.albedo_map = load_tex(pbr.base_color_texture());

    mat
}

#[derive(Debug, Copy, Clone)]
pub struct Untextured;

pub type UntexturedMat = Material<Untextured>;

const fn untextured_mat(
    albedo_factor: Vec4,
    metallic_factor: f32,
    roughness_factor: f32,
) -> UntexturedMat {
    Material {
        albedo_factor,
        metallic_factor,
        roughness_factor,
        emissive_factor: Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        albedo_map: None,
        metallic_roughness_map: None,
        emissive_map: None,
        normal_map: None,
        occlusion_map: None,
    }
}

pub mod base {
    use super::*;

    pub const GOLD: UntexturedMat = untextured_mat(
        Vec4 {
            x: 1.0,
            y: 0.766,
            z: 0.336,
            w: 1.0,
        },
        1.0,
        0.3,
    );
    pub const PLASTIC_WHITE: UntexturedMat = untextured_mat(
        Vec4 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
            w: 1.0,
        },
        0.0,
        0.3,
    );
    pub const PLASTIC_RED: UntexturedMat = Material {
        albedo_factor: Vec4 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        },
        ..PLASTIC_WHITE
    };

}
