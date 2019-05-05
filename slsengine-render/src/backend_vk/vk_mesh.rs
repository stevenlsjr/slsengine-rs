use super::VulkanRenderer;
use crate::::{Mesh, RenderMesh, Vertex};
use failure as f;
use std::sync::Arc;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer, DeviceLocalBuffer},
    command_buffer::{AutoCommandBufferBuilder, CommandBuffer},
    device::Device,
    sync::GpuFuture,
};

#[derive(Debug)]
pub struct VkMesh {
    mesh: Mesh,
    pub vertex_buffer: Arc<DeviceLocalBuffer<[Vertex]>>,
    pub index_buffer: Arc<DeviceLocalBuffer<[u32]>>,
}

impl VkMesh {
    pub fn new(
        renderer: &VulkanRenderer,
        mesh: Mesh,
    ) -> Result<VkMesh, f::Error> {
        let device = renderer.device.clone();
        let staging_queue = renderer.queues.graphics_queue.clone();
        let staging_vbo = CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            mesh.vertices.clone().into_iter(),
        )
        .map_err(&f::Error::from)?;
        let staging_ibo: Arc<CpuAccessibleBuffer<[u32]>> =
            CpuAccessibleBuffer::from_iter(
                device.clone(),
                BufferUsage::all(),
                mesh.indices.clone().into_iter(),
            )
            .map_err(&f::Error::from)?;

        let vertices = DeviceLocalBuffer::<[Vertex]>::array(
            device.clone(),
            mesh.vertices.len(),
            BufferUsage::all(),
            Some(staging_queue.family()),
        )
        .map_err(&f::Error::from)?;
        let indices = DeviceLocalBuffer::<[u32]>::array(
            device.clone(),
            mesh.indices.len(),
            BufferUsage::all(),
            Some(staging_queue.family()),
        )
        .map_err(&f::Error::from)?;

        let mut cbb = AutoCommandBufferBuilder::new(
            device.clone(),
            staging_queue.family(),
        )?;
        cbb = cbb.copy_buffer(staging_ibo, indices.clone())?;
        cbb = cbb.copy_buffer(staging_vbo, vertices.clone())?;
        let cb = cbb.build()?;
        let fut = cb.execute(staging_queue.clone())?;
        (fut.then_signal_fence_and_flush()?).wait(None).unwrap();
        Ok(VkMesh {
            vertex_buffer: vertices,
            index_buffer: indices,
            mesh,
        })
    }
}

impl RenderMesh for VkMesh {
    fn mesh(&self) -> &Mesh {
        &self.mesh
    }
}
