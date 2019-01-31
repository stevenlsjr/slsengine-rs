use super::component::*;
use super::resource::{MeshHandle, ResourceManager, TextureHandle};
use crate::math::*;
use crate::renderer::{material::Material, mesh::RenderMesh};
use cgmath::*;
use std::fmt::{self, Debug};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct TransformComponent {
    pub parent: Option<Entity>,
    pub transform: Decomposed<Vec3, Quaternion<f32>>,
}

impl Component for TransformComponent {
    // const MASK: ComponentMask = ComponentMask::TRANSFORM;
}

impl Default for TransformComponent {
    fn default() -> Self {
        TransformComponent {
            parent: None,
            transform: Decomposed {
                scale: 1.0,
                rot: Quaternion::zero(),
                disp: Vec3::zero(),
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct MaterialComponent {
    pub material: Material<TextureHandle>,
}

impl Component for MaterialComponent {}

#[derive(Debug, Clone)]
pub struct MeshComponent {
    pub mesh: MeshHandle,
}

impl Component for MeshComponent {
    // const MASK: ComponentMask = ComponentMask::MESH;
}
