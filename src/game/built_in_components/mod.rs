use super::component::*;
use crate::math::*;
use crate::renderer::{material::Material, mesh::RenderMesh};
use cgmath::*;
use std::fmt::{self, Debug};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct TransformComponent {
    pub parent: Option<EntityId>,
    pub transform: Decomposed<Vec3, Quaternion<f32>>,
}

impl Component for TransformComponent {
    const MASK: ComponentMask = ComponentMask::TRANSFORM;
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

#[derive(Clone)]
pub struct MaterialComponent<Tex> {
    material: Material<Tex>,
}
impl<T> fmt::Debug for MaterialComponent<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MaterialComponent {{material: {:?}}}", self.material)
    }
}

impl<Tex> Component for MaterialComponent<Tex> {
    const MASK: ComponentMask = ComponentMask::MATERIAL;
}

#[derive(Clone)]
pub struct MeshComponent<M>
where
    M: RenderMesh,
{
    pub name: String,
    pub mesh: Option<Arc<M>>,
}



impl<M> Component for MeshComponent<M>
where
    M: RenderMesh,
{
    const MASK: ComponentMask = ComponentMask::STATIC_MESH;
}
impl<M> Debug for MeshComponent<M>
where
    M: RenderMesh,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MeshComponent {}", self.name)
    }
}
