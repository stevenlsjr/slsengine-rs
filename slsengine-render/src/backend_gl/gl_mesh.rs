use super::objects::*;
use crate::*;
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
    fn mesh(&self) -> &Mesh {
        &self.mesh
    }
}
