use cgmath::*;
use math::*;

#[derive(Debug, Clone)]
pub struct Material<Tex> {
    pub albedo_factor: Vec4,
    pub albedo_map: Option<Tex>,
    pub metallic_factor: f32,
    pub metallic_map: Option<Tex>,
    pub roughness_factor: f32,
    pub roughness_map: Option<Tex>,
    pub emissive_factor: Vec3,
    pub emissive_map: Option<Tex>,
    pub normal_map: Option<Tex>,
    pub occlusion_map: Option<Tex>,
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
        F: Fn(&Tex) -> BTex,
    {
        let f = &f;
        let mut mat: Material<BTex> = Material::default();
        mat.albedo_map = self.albedo_map.as_ref().map(f);
        mat.metallic_map = self.metallic_map.as_ref().map(f);
        mat.roughness_map = self.roughness_map.as_ref().map(f);
        mat.emissive_map = self.emissive_map.as_ref().map(f);
        mat.occlusion_map = self.occlusion_map.as_ref().map(f);
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
    let m2 = m.transform_textures(|tex| tex * 2);
    assert_eq!(m2.albedo_map, Some(2));
    assert_eq!(m2.occlusion_map, None);
}

impl<Tex> Default for Material<Tex> {
    fn default() -> Self {
        Material {
            albedo_factor: vec4(1.0, 1.0, 1.0, 1.0),
            albedo_map: None,
            metallic_factor: 0.0,
            metallic_map: None,
            roughness_factor: 1.0,
            roughness_map: None,
            emissive_factor: vec3(0.0, 0.0, 0.0),
            emissive_map: None,
            normal_map: None,
            occlusion_map: None,
        }
    }
}

pub struct Untextured;

pub type UntexturedMat = Material<Untextured>;
pub mod base {
    use super::*;
   
    pub const GOLD: UntexturedMat = Material {
        albedo_factor: Vec4 {
            x: 1.0,
            y: 0.766,
            z: 0.336,
            w: 1.0,
        },

        metallic_factor: 1.0,

        roughness_factor: 0.3,

        emissive_factor: Vec3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
        albedo_map: None,
        metallic_map: None,
        roughness_map: None,
        emissive_map: None,
        normal_map: None,
        occlusion_map: None,
    };

}
