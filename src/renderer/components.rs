use specs::prelude::*;

use cgmath::*;

#[derive(Debug, Component)]
pub struct TransformComponent {
    pub transform: Decomposed<Vector3<f32>, Quaternion<f32>>
}

#[derive(Debug, Component)]
pub struct MeshComponent{

}

