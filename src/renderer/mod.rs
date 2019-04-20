#[cfg(feature = "backend-gl")]
pub mod backend_gl;
#[cfg(feature = "backend-vulkan")]
pub mod backend_vk;
pub mod camera;
pub mod color;
pub mod material;
pub mod mesh;
pub mod model;
pub mod traits;
pub mod components;

pub use self::color::{color4f, ColorRGBA};
pub use self::{camera::*, mesh::*, traits::*};

pub trait ShaderPipeline<T: Renderer> {
    fn use_program(&self, renderer: &T);
}


