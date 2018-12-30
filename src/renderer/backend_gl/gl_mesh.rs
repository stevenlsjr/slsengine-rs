use super::objects::*;
use crate::renderer::*;
#[derive(Debug)]
pub struct GlMesh {
    pub mesh: Mesh,
    pub buffers: MeshBuffers,
}

impl RenderMesh for GlMesh {
    fn vertices(&self) -> &[Vertex] {
        self.mesh.vertices()
    }
    fn indices(&self) -> &[u32] {
        self.mesh.indices()
    }
}
