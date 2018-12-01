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
