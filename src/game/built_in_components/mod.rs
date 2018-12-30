use super::component::*;
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
    pub material: Material<Tex>,
}
impl<T> fmt::Debug for MaterialComponent<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MaterialComponent {{material: {:?}}}", self.material)
    }
}

impl<Tex> Component for MaterialComponent<Tex> {
    const MASK: ComponentMask = ComponentMask::MATERIAL;
}

#[derive(Debug)]
pub struct MeshComponent<M>
where
    M: RenderMesh + Debug,
{
    pub mesh: Arc<M>,
}
impl<M> Clone for MeshComponent<M>
where
    M: RenderMesh + Debug,
{
    fn clone(&self) -> Self {
        MeshComponent {
            mesh: self.mesh.clone(),
        }
    }
}
