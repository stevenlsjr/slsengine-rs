pub mod backend_gl;
#[cfg(feature = "with-vulkan")]
pub mod backend_vk;
pub mod camera;
pub mod material;
pub mod mesh;
pub mod model;
pub mod traits;


pub use self::{camera::*, mesh::*, traits::*};

pub trait ShaderPipeline<T: Renderer> {
    fn use_program(&self, renderer: &T);
}
