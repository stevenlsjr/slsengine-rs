// use super::ash;

use cgmath;
use failure;
use renderer::*;
use sdl2;
use sdl2::video::Window;
use sdl2::video::{Window as SdlWindow, WindowContext};
use std::cell::{Ref, RefCell};
use std::default::Default;
use std::ffi::{CStr, CString};
use std::rc::Rc;
use std::sync::Arc;
use std::{mem, ptr};
use vulkano;
use vulkano::device::{Device, Queue};
use vulkano::image::SwapchainImage;
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::swapchain::{
    Capabilities, ColorSpace, CompositeAlpha, PresentMode,
    SupportedPresentModes, Surface, Swapchain,
};

use std::fmt;

pub mod sdl_vulkan;

pub type VulkanWinType = Rc<WindowContext>;
pub type SdlSurface = Surface<VulkanWinType>;

/// Error Record for VkContext creation
#[derive(Fail, Clone, Debug)]
pub enum VkContextError {
    #[fail(display = "could not create entry")]
    Entry,
    #[fail(display = "could not create instance")]
    Instance,
    #[fail(display = "could not load {} extensions", _0)]
    Extensions(String),
    #[fail(display = "could not load Vulkan SurfaceKHR functions")]
    SurfaceLoader,
    #[fail(display = "could not find Queue Families")]
    QueueFamilies,
    #[fail(display = "could not create surfaceKHR")]
    Surface,
    #[fail(display = "could not create device")]
    Device,

    #[fail(display = "Could not create vulkan rendering context: {:#?}", _0)]
    Other(String),
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
        let rating = rate_physical_device(dev);
        if top_device.1.is_none() || rating > top_device.0 {
            top_device = (rating, Some(dev));
        }

        println!("device {:?} {:?} with rating {}", dev, dev.name(), rating);
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
    fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
    fn build(&self) -> Option<QueueFamilies> {
        self.graphics_family.and_then(|graphics_family| {
            self.present_family.map(|present_family| QueueFamilies {
                graphics_family,
                present_family,
            })
        })
    }
}

pub struct QueueFamilies {
    graphics_family: u32,
    present_family: u32,
}

impl QueueFamilies {
    pub fn new<W>(
        instance: &Instance,
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
    use vulkano::device::{DeviceExtensions, Features};
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
    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        //        khr_display_swapchain: true,
        ..DeviceExtensions::none()
    };

    let (device, mut queues) = Device::new(
        *physical_device,
        &Features::none(),
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

fn create_swapchain(
    instance: &Arc<Instance>,
    surface: &Arc<Surface<VulkanWinType>>,
    physical_device: &PhysicalDevice,
    device: &Arc<Device>,
    queues: &VulkanQueues,
) -> Result<(), failure::Error> {
    let capabilities = surface
        .capabilities(*physical_device)
        .map_err(&failure::Error::from)?;

    unimplemented!()
}

/// Renderer object for vulkan context
pub struct VulkanRenderer {
    pub camera: RefCell<Camera>,
    pub instance: Arc<Instance>,
    pub device: Arc<Device>,
    pub queues: VulkanQueues,
    pub surface: Arc<Surface<VulkanWinType>>,
    pub swapchain: Arc<Swapchain<VulkanWinType>>,
    pub swapchain_image: Arc<SwapchainImage<VulkanWinType>>,
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
        let layers = vec!["VK_LAYER_LUNARG_standard_validation"];
        let instance = Instance::new(None, raw_instance_extensions, layers)
            .expect("failed to create vulkan instance");
        let surface: Arc<SdlSurface> = {
            let handle = window
                .vulkan_create_surface(instance.internal_object())
                .map_err(|_| VkContextError::Surface)?;
            unsafe {
                Arc::new(Surface::from_raw_surface(
                    instance.clone(),
                    handle,
                    window.context().clone(),
                ))
            }
        };
        let physical_device = pick_physical_device(&instance)
            .map_err(|_| VkContextError::Device)?;
        let (device, queues) = create_device(&physical_device, &surface)
            .map_err(|_| VkContextError::Device)?;
        let swapchain = create_swapchain(
            &instance,
            &surface,
            &physical_device,
            &device,
            &queues,
        )
        .map_err(|e| VkContextError::Other(e.to_string()));
        unimplemented!();
        //        Ok(VulkanRenderer {
        //            camera: RefCell::new(Camera::new(cgmath::PerspectiveFov {
        //                fovy: cgmath::Deg(40.0).into(),
        //                aspect: 1.0,
        //                near: 0.1,
        //                far: 100.0,
        //            })),
        //            instance: instance.clone(),
        //            device,
        //            queues,
        //            surface,
        //        })
    }
}
pub struct VkTexture;
pub struct VkMesh;

impl Renderer for VulkanRenderer {
    type Texture = VkTexture;
    type Mesh = VkMesh;
    fn camera(&self) -> Ref<Camera> {
        self.camera.borrow()
    }
}
