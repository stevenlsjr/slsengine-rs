pub mod pipelines;
/// vulkan-specific SDL platform utilities
pub mod sdl_vulkan;
pub mod shaders;
pub mod vulkan_renderer;

use super::mesh::Vertex;
use crate::renderer::*;
use cgmath;
use failure;
use log::*;
use sdl2;
use sdl2::video::{Window, WindowContext};
use std::{
    cell::{Ref, RefCell},
    ffi::CString,
    fmt,
    rc::Rc,
    sync::{Arc, RwLock},
};
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

pub use self::sdl_vulkan::VulkanPlatformHooks;
pub type VulkanWinType = Rc<WindowContext>;
pub type SdlSurface = Surface<VulkanWinType>;
pub type SdlSwapchain = Swapchain<VulkanWinType>;
pub type SdlSwapchainImage = SwapchainImage<VulkanWinType>;
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

fn rate_physical_device(phys_dev: PhysicalDevice) -> i32 {
    use vulkano::instance::PhysicalDeviceType;
    let mut score = 0;
    let dev_features = phys_dev.supported_features();

    for &feature in [dev_features.geometry_shader].iter() {
        if feature {
            score += 100
        }
    }

    match phys_dev.ty() {
        PhysicalDeviceType::DiscreteGpu => score += 1000,
        PhysicalDeviceType::IntegratedGpu => score += 500,
        _ => {}
    }
    score
}

pub fn pick_physical_device(
    instance: &Arc<Instance>,
) -> Result<PhysicalDevice, failure::Error> {
    let mut top_device = (0, None::<PhysicalDevice>);
    for dev in PhysicalDevice::enumerate(instance) {
        if !device_extensions_supported(&dev) {
            continue;
        }
        let rating = rate_physical_device(dev);
        if top_device.1.is_none() || rating > top_device.0 {
            top_device = (rating, Some(dev));
        }

        info!("device {:?} {:?} with rating {}", dev, dev.name(), rating);
    }
    match top_device.1 {
        Some(dev) => Ok(dev),
        None => Err(format_err!("could not find a suitible physical device")),
    }
}

struct QueueFamilyBuilder {
    graphics_family: Option<u32>,
    present_family: Option<u32>,
}

impl QueueFamilyBuilder {
    fn build(&self) -> Option<QueueFamilies> {
        match (self.graphics_family, self.present_family) {
            (Some(graphics_family), Some(present_family)) => {
                Some(QueueFamilies {
                    graphics_family,
                    present_family,
                })
            }
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct QueueFamilies {
    graphics_family: u32,
    present_family: u32,
}

impl QueueFamilies {
    pub fn new<W>(
        _instance: &Instance,
        physical_device: &PhysicalDevice,
        surface: &Surface<W>,
    ) -> Result<QueueFamilies, failure::Error> {
        let mut builder = QueueFamilyBuilder {
            graphics_family: None,
            present_family: None,
        };
        for family in physical_device.queue_families() {
            let is_graphics_queue =
                family.supports_compute() && family.supports_graphics();
            if family.queues_count() > 0 && is_graphics_queue {
                builder.graphics_family = Some(family.id());
            }

            if surface.is_supported(family).expect(&format!(
                "{:#?} could not check for queue family support!",
                surface
            )) {
                builder.present_family = Some(family.id())
            }

            if let Some(families) = builder.build() {
                return Ok(families);
            }
        }

        bail!("valid queue families not found")
    }
}

#[derive(Clone, Debug)]
pub struct VulkanQueues {
    pub present_queue: Arc<Queue>,
    pub graphics_queue: Arc<Queue>,
}

pub fn create_device<W>(
    physical_device: &PhysicalDevice,
    surface: &Surface<W>,
) -> Result<(Arc<Device>, VulkanQueues), failure::Error> {
    let instance = physical_device.instance();
    let queue_families = QueueFamilies::new(instance, physical_device, surface)
        .map_err(|e| format_err!("could not find queue families: {:#?}", e))?;
    use std::collections::HashSet;
    use std::iter::FromIterator;
    let unique_queue_families: HashSet<u32> = HashSet::from_iter(
        [
            queue_families.graphics_family,
            queue_families.present_family,
        ]
        .iter()
        .cloned(),
    );

    let default_queue_priority = 1.0;
    let queue_families = unique_queue_families.iter().filter_map(|&i| {
        physical_device
            .queue_family_by_id(i)
            .map(|fam| (fam, default_queue_priority as f32))
    });
    let device_extensions = required_device_extensions();
    let features = physical_device.supported_features();

    let (device, mut queues) = Device::new(
        *physical_device,
        features,
        &device_extensions,
        queue_families,
    )
    .map_err(&failure::Error::from)?;

    let graphics_queue = queues.next().unwrap();
    let present_queue = queues.next().unwrap_or_else(|| graphics_queue.clone());
    Ok((
        device,
        VulkanQueues {
            graphics_queue,
            present_queue,
        },
    ))
}

fn required_device_extensions() -> DeviceExtensions {
    DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::none()
    }
}

fn device_extensions_supported(dev: &PhysicalDevice) -> bool {
    use vulkano::device::DeviceExtensions;
    let availible_extensions = DeviceExtensions::supported_by_device(*dev);
    let device_extensions = required_device_extensions();
    availible_extensions.intersection(&device_extensions) == device_extensions
}

fn pick_surface_format(capabilities: &Capabilities) -> (Format, ColorSpace) {
    use vulkano::format::Format;

    *capabilities
        .supported_formats
        .iter()
        .find(|(format, color_space)| {
            *format == Format::B8G8R8A8Unorm
                && *color_space == ColorSpace::SrgbNonLinear
        })
        .unwrap_or(&capabilities.supported_formats[0])
}

fn create_swapchain(
    _instance: &Arc<Instance>,
    surface: &Arc<Surface<VulkanWinType>>,
    physical_device: &PhysicalDevice,
    device: &Arc<Device>,
    queues: &VulkanQueues,
) -> Result<(Arc<SdlSwapchain>, Vec<Arc<SdlSwapchainImage>>), failure::Error> {
    use vulkano::swapchain::SurfaceTransform;

    let capabilities = surface.capabilities(*physical_device)?;
    let (format, _colorspace) = pick_surface_format(&capabilities);
    let usage = capabilities.supported_usage_flags;
    let alpha = capabilities
        .supported_composite_alpha
        .iter()
        .next()
        .unwrap();

    let size = {
        let window: &Window =
            unsafe { &Window::from_ref(surface.window().clone()) };
        window.vulkan_drawable_size()
    };
    let sharing: SharingMode =
        if queues.graphics_queue.is_same(&queues.present_queue) {
            (&queues.graphics_queue).into()
        } else {
            let slice: &[&Arc<Queue>] =
                &[&queues.graphics_queue, &queues.present_queue];
            slice.into()
        };

    Swapchain::new(
        device.clone(),
        surface.clone(),
        capabilities.min_image_count,
        format,
        [size.0, size.1],
        1,
        usage,
        sharing,
        SurfaceTransform::Identity,
        alpha,
        PresentMode::Fifo,
        true,
        None,
    )
    .map_err(&failure::Error::from)
}

fn create_renderpass(
    device: Arc<Device>,
    swapchain: &Arc<SdlSwapchain>,
) -> Result<Arc<RenderPassAbstract + Send + Sync>, VkContextError> {
    let renderpass = single_pass_renderpass! (
    device,
    attachments: {
        out_color: {load: Clear, store: Store, format: swapchain.format(),
        samples: 1,}
    },
    pass: {color: [out_color],
    depth_stencil: {}}
    )
    .map_err(|e| VkContextError::component_creation("render_pass", Some(e)))?;

    Ok(Arc::new(renderpass))
}

/// Renderer object for vulkan context
pub struct VulkanRenderer {
    pub camera: RefCell<Camera>,
    pub instance: Arc<Instance>,
    pub device: Arc<Device>,
    pub queues: VulkanQueues,
    pub surface: Arc<Surface<VulkanWinType>>,
    pub swapchain: Arc<Swapchain<VulkanWinType>>,
    pub swapchain_images: Vec<Arc<SwapchainImage<VulkanWinType>>>,
    pub render_pass: Arc<RenderPassAbstract + Send + Sync>,
    pub dynamic_state: RwLock<DynamicState>,
    pub pipelines: pipelines::RendererPipelines,
}
impl fmt::Debug for VulkanRenderer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VulkanRenderer {:?}", self.device)
    }
}

impl VulkanRenderer {
    /// Creates a new Renderer from an SDL window
    pub fn new(window: &Window) -> Result<Self, VkContextError> {
        use vulkano::instance::{
            Instance, PhysicalDevice, RawInstanceExtensions,
        };
        use vulkano::swapchain::Surface;
        use vulkano::VulkanObject;
        let instance_extensions = window.vulkan_instance_extensions().unwrap();
        let raw_instance_extensions = RawInstanceExtensions::new(
            instance_extensions
                .iter()
                .map(|&v| CString::new(v).unwrap()),
        );
        let layers = if cfg!(feature = "gl-debug-output") {
            vec!["VK_LAYER_LUNARG_standard_validation"]
        } else {
            Vec::new()
        };
        let instance = Instance::new(None, raw_instance_extensions, layers)
            .expect("failed to create vulkan instance");
        let surface: Arc<SdlSurface> = {
            let handle = window
                .vulkan_create_surface(instance.internal_object())
                .map_err(|e| {
                    VkContextError::component_creation(
                        "surface",
                        Some(failure::err_msg(e)),
                    )
                })?;
            unsafe {
                Arc::new(Surface::from_raw_surface(
                    instance.clone(),
                    handle,
                    window.context().clone(),
                ))
            }
        };
        let physical_device = pick_physical_device(&instance).map_err(|e| {
            VkContextError::component_creation("physical device", Some(e))
        })?;
        let (device, queues) = create_device(&physical_device, &surface)
            .map_err(|e| {
                VkContextError::component_creation("device", Some(e))
            })?;
        let (swapchain, images) = create_swapchain(
            &instance,
            &surface,
            &physical_device,
            &device,
            &queues,
        )
        .map_err(|e| {
            VkContextError::component_creation("swapchain", Some(e))
        })?;

        let render_pass = create_renderpass(device.clone(), &swapchain)?;

        let pipelines = pipelines::RendererPipelines::new(&device, &render_pass)?;
        Ok(VulkanRenderer {
            camera: RefCell::new(Camera::new(cgmath::PerspectiveFov {
                fovy: cgmath::Deg(40.0).into(),
                aspect: 1.0,
                near: 0.1,
                far: 100.0,
            })),
            instance: instance.clone(),
            device,
            queues,
            surface,
            swapchain,
            swapchain_images: images,
            render_pass,
            pipelines,
            dynamic_state: RwLock::new(DynamicState::none()),
        })
    }
}
#[derive(Debug)]
pub struct VkTexture;

impl Renderer for VulkanRenderer {
    type Texture = Arc<VkTexture>;
    type Mesh = Mesh;
    fn camera(&self) -> Ref<Camera> {
        self.camera.borrow()
    }

    fn on_resize(&self, _size: (u32, u32)) {}
}
