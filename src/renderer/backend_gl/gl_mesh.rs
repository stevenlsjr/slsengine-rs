use super::objects::*;
use crate::renderer::*;
#[derive(Debug)]
pub struct GlMesh {
    pub mesh: Mesh,
    pub buffers: MeshBuffers,
}

impl GlMesh {
    pub fn with_mesh(mesh: Mesh) -> Result<Self, failure::Error> {
        let mut buffers = MeshBuffers::new()?;
        buffers.bind_mesh(&mesh)?;
        Ok(GlMesh { mesh, buffers })
    }
}

impl RenderMesh for GlMesh {
    fn vertices(&self) -> &[Vertex] {
        self.mesh.vertices()
    }
    fn indices(&self) -> &[u32] {
        self.mesh.indices()
    }
}
