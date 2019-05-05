use specs::prelude::*;
use specs::storage::*;

use crate::mesh::Mesh;
use cgmath::*;
use slsengine_entityalloc::allocator::GenerationalIndex;

#[derive(Debug, Component)]
pub struct TransformComponent {
    pub transform: Decomposed<Vector3<f32>, Quaternion<f32>>,
}

impl Default for TransformComponent {
    fn default() -> Self {
        TransformComponent {
            transform: Decomposed::one(),
        }
    }
}

#[derive(Debug, Component, SmartDefault)]
#[storage(BTreeStorage)]
pub struct MeshComponent {
    mesh: Option<Mesh>,
    renderer_asset: Option<GenerationalIndex>,
}
