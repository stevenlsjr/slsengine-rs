use crate::renderer::{Mesh, Vertex};
use std::sync::Arc;
use vulkano::buffer::*;

pub struct VkMesh {
    mesh: Mesh,
    vertex_buffer: Arc<DeviceLocalBuffer<[Vertex]>>,
    index_buffer: Arc<DeviceLocalBuffer<[u32]>>,
}

