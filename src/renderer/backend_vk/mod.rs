pub mod pipelines;
/// vulkan-specific SDL platform utilities
pub mod sdl_vulkan;
pub mod shaders;
pub mod vulkan_renderer;

pub mod vk_mesh;

pub use self::sdl_vulkan::VulkanPlatformHooks;
pub use self::vulkan_renderer::{VkTexture, VulkanQueues, VulkanRenderer};
pub use self::vk_mesh::VkMesh;

use super::mesh::Vertex;
use cgmath;
use failure;
use sdl2;
use std::sync::{Arc, RwLock};
use vulkano::{
    self,
    buffer::*,
    command_buffer::*,
    descriptor::descriptor_set::*,
    device::*,
    format::*,
    framebuffer::*,
    image::*,
    impl_vertex,
    instance::*,
    pipeline::{viewport::*, *},
    single_pass_renderpass,
    swapchain::*,
    sync::*,
};

#[allow(clippy::ref_in_deref)]
vulkano::impl_vertex!(Vertex, position);
pub type VkResult<T> = Result<T, VkContextError>;

/// Type that vulkano objects use for tracking window for Surface<T> and
/// types referencing it. Uses an empty tuple instead of a Window
/// because SDL window handles are not thread-safe. For now, it will be the
/// responsibility of the user to ensure the SDL window lasts as long
/// as any vulkan surfaces.
pub type VulkanWinType = ();
pub type SdlSurface = Surface<VulkanWinType>;
pub type SdlSwapchain = Swapchain<VulkanWinType>;
pub type SdlSwapchainImage = SwapchainImage<VulkanWinType>;

pub type DynGraphicsPipeline = Arc<dyn GraphicsPipelineAbstract + Send + Sync>;
pub type DynRenderPass = Arc<dyn RenderPassAbstract + Send + Sync>;
pub type DynFramebuffer = Arc<dyn FramebufferAbstract + Send + Sync>;
/// Error Record for VkContext creation
#[derive(Fail, Debug)]
pub enum VkContextError {
    #[fail(display = "could not create entry")]
    Entry,
    #[fail(display = "could not create instance")]
    Instance,
    #[fail(display = "could not load {} extensions", _0)]
    Extensions(String),
    #[fail(display = "could not load Vulkan SurfaceKHR functions")]
    SurfaceLoader,

    #[fail(
        display = "could not create renederer component: {}, {:?}",
        component, cause
    )]
    ComponentCreation {
        component: String,
        cause: Option<failure::Error>,
    },

    #[fail(display = "Could not create vulkan rendering context: {:#?}", _0)]
    Other(String),
    #[fail(display = "could not create renderer, error caused by {:?}", _0)]
    OtherError(failure::Error),
}

impl VkContextError {
    pub fn other_error<E: failure::Fail>(error: E) -> VkContextError {
        VkContextError::OtherError(failure::Error::from(error))
    }
    pub fn component_creation<E: Into<failure::Error> + Send + Sync>(
        comp_name: &str,
        cause: Option<E>,
    ) -> VkContextError {
        VkContextError::ComponentCreation {
            component: comp_name.to_owned(),
            cause: cause.map(|e| e.into()),
        }
    }
}
