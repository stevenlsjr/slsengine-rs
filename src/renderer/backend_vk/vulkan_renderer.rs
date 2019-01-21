use super::*;
use crate::{game::{prelude::*, EntityWorld, component::*, built_in_components::*}, math::*, renderer::mesh::*, renderer::*};
use cgmath::*;
use failure;
use log::*;
use sdl2;
use sdl2::video::Window;
use std::{
    cell::{Ref, RefCell},
    ffi::CString,
    fmt,
    rc::Rc,
    ops::Try,
    sync::{atomic::*, Arc, RwLock},
};
use vulkano::{
    self, buffer::*, command_buffer::*, descriptor::descriptor_set::*,
    device::*, format::*, framebuffer::*, image::*, instance::*,
    pipeline::viewport::*, single_pass_renderpass, swapchain::*, sync::*,
};

use slsengine_entityalloc::IndexArray;

struct Foo {
    f: FenceSignalFuture<Box<GpuFuture>>,
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

            builder.present_family = surface
                .is_supported(family)
                .map(|b| if b { Some(family.id()) } else { None })
                .unwrap_or_else(|e| {
                    panic!(
                        "{:#?} could not check for queue family support! {:?}",
                        surface, e
                    )
                });

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
    window: &Window,
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

    let size = window.vulkan_drawable_size();
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

/// Helper type for constructing vulkan context.
struct Builder<'a> {
    // Resources used in context build steps. Not builder params per-se
    // Instead, more like a traditional nullable+mutable initialization pattern
    window: &'a Window,
    instance: Option<Arc<Instance>>,
    device: Option<Arc<Device>>,
    queues: Option<VulkanQueues>,
    surface: Option<Arc<SdlSurface>>,
    swapchain: Option<Arc<SdlSwapchain>>,
    swapchain_images: Option<Vec<Arc<SdlSwapchainImage>>>,
    render_pass: Option<Arc<RenderPassAbstract + Send + Sync>>,
    dynamic_state: Option<RwLock<DynamicState>>,
    pipelines: Option<pipelines::RendererPipelines>,
}

impl<'a> Builder<'a> {
    fn new(window: &'a Window) -> Self {
        Builder {
            window,
            instance: None,
            device: None,
            queues: None,
            surface: None,
            swapchain: None,
            swapchain_images: None,
            render_pass: None,
            dynamic_state: None,
            pipelines: None,
        }
    }

    fn build(mut self) -> Result<VulkanRenderer, VkContextError> {
        let window = self.window;
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
            .map_err(|e| {
                VkContextError::component_creation("instance", Some(e))
            })?;
        self.instance = Some(instance.clone());
        self.create_surface()?;

        self.create_device()?;
        self.create_swapchain(self.window)?;

        self.create_renderpass()?;
        {
            let device = self.device.unwrap();

            let render_pass = self.render_pass.as_ref().unwrap();

            let pipelines =
                pipelines::RendererPipelines::new(&device, &render_pass)?;

            let previous_frame_end = RefCell::new(None);
            let aspect = {
                let (width, height) = window.size();
                (width as f32) / (height as f32)
            };

            Ok(VulkanRenderer {
                camera: RefCell::new(Camera::new(cgmath::PerspectiveFov {
                    fovy: cgmath::Deg(40.0).into(),
                    aspect: aspect,
                    near: 0.1,
                    far: 100.0,
                })),
                instance,
                device,
                queues: self.queues.unwrap(),
                surface: self.surface.unwrap(),
                pipelines,

                render_pass: self.render_pass.unwrap(),
                recreate_swapchain: AtomicBool::new(false),
                previous_frame_end,

                state: RefCell::new(RenderingState {
                    swapchain: self.swapchain.unwrap(),
                    swapchain_images: self.swapchain_images.unwrap(),
                    framebuffers: Vec::new(),
                    dynamic_state: DynamicState::none(),
                }),
            })
        }
    }

    fn create_device(&mut self) -> Result<(), VkContextError> {
        if let (Some(surface), Some(instance)) = (&self.surface, &self.instance)
        {
            let physical_device =
                pick_physical_device(&instance).map_err(|e| {
                    VkContextError::component_creation(
                        "physical device",
                        Some(e),
                    )
                })?;
            let (device, queues) = create_device(&physical_device, &surface)
                .map_err(|e| {
                    VkContextError::component_creation("device", Some(e))
                })?;

            self.device = Some(device);
            self.queues = Some(queues);

            Ok(())
        } else {
            panic!("device, surface, and instance must first be created");
        }
    }
    fn create_surface(&mut self) -> Result<(), VkContextError> {
        let window = self.window;
        use vulkano::VulkanObject;
        let instance = self
            .instance
            .clone()
            .expect("instance should have already been created");
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
                    (),
                ))
            }
        };
        self.surface = Some(surface);
        Ok(())
    }

    fn create_swapchain(
        &mut self,
        window: &Window,
    ) -> Result<(), VkContextError> {
        let instance = self.instance.as_ref().unwrap();
        let device = self.device.as_ref().unwrap();
        let physical_device = device.physical_device();
        let queues = self.queues.as_ref().unwrap();
        let surface = self.surface.as_ref().unwrap();
        let (swapchain, images) = create_swapchain(
            &instance,
            surface,
            &physical_device,
            device,
            queues,
            window,
        )
        .map_err(|e| {
            VkContextError::component_creation("swapchain", Some(e))
        })?;

        self.swapchain = Some(swapchain);
        self.swapchain_images = Some(images);

        Ok(())
    }

    fn create_renderpass(&mut self) -> Result<(), VkContextError> {
        let swapchain = self.swapchain.as_ref().unwrap();
        let device = self.device.as_ref().unwrap();
        let format = swapchain.format();
        let renderpass = Arc::new(
            single_pass_renderpass!(
            device.clone(),
            attachments: {
                out_color: {
                    load: Clear,
                    store: Store,
                    format: format,
                    samples: 1,
                    },
                depth: {
                    load: Clear,
                    store: DontCare,
                    format: Format::D16Unorm,
                    samples: 1,
                }
            },
            pass: {color: [out_color],
            depth_stencil: {depth}}
            )
            .map_err(|e| {
                VkContextError::component_creation("render_pass", Some(e))
            })?,
        );
        self.render_pass = Some(renderpass);
        Ok(())
    }
}

/// Renderer object for vulkan context
pub struct VulkanRenderer {
    pub camera: RefCell<Camera>,
    pub instance: Arc<Instance>,
    pub device: Arc<Device>,
    pub queues: VulkanQueues,
    pub surface: Arc<SdlSurface>,
    pub pipelines: pipelines::RendererPipelines,
    pub render_pass: Arc<RenderPassAbstract + Send + Sync>,
    /// or future for synchronizing last frame end
    recreate_swapchain: AtomicBool,
    previous_frame_end: RefCell<Option<FenceSignalFuture<Box<dyn GpuFuture>>>>,
    /// lock for managing resources replaced during program's progress, such as flagging swapchain recreation
    pub state: RefCell<RenderingState>,
}

pub struct RenderingState {
    pub swapchain: Arc<SdlSwapchain>,
    pub swapchain_images: Vec<Arc<SdlSwapchainImage>>,
    pub dynamic_state: DynamicState,
    pub framebuffers: Vec<DynFramebuffer>,
}

impl fmt::Debug for VulkanRenderer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VulkanRenderer {:?}", self.device)
    }
}

impl VulkanRenderer {
    /// Creates a new Renderer from an SDL window
    pub fn new(window: &Window) -> Result<Self, VkContextError> {
        let builder = Builder::new(window);
        let renderer = builder.build()?;
        renderer.window_size_fb_setup()?;
        Ok(renderer)
    }

    fn window_size_fb_setup(&self) -> VkResult<()> {
        let (viewport, new_fbs) = {
            let state = self.state.borrow();
            let images = &state.swapchain_images;
            let dimensions = SdlSwapchainImage::dimensions(&images[0]);

            let viewport = Viewport {
                origin: [0.0, 0.0],
                dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                depth_range: 0.0..1.0,
            };
            {
                let mut perspective = self.camera().perspective();
                perspective.aspect =
                    dimensions[0] as f32 / dimensions[1] as f32;
                self.camera.replace(Camera::new(perspective));
            }

            let depth_buffer = AttachmentImage::transient(
                self.device.clone(),
                dimensions,
                Format::D16Unorm,
            )
            .map_err(|e| {
                VkContextError::component_creation(
                    "depth image buffer",
                    Some(e),
                )
            })?;
            let new_fbs = {
                images
                    .iter()
                    .map(|image| {
                        Framebuffer::start(self.render_pass.clone())
                            .add(image.clone())
                            .and_then(|fb| fb.add(depth_buffer.clone()))
                            .and_then(|fb| fb.build())
                            .map(|fb| Arc::new(fb) as DynFramebuffer)
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| {
                        VkContextError::component_creation(
                            "framebuffer creation",
                            Some(e),
                        )
                    })?
            };
            (viewport, new_fbs)
        };
        {
            let mut state = self.state.borrow_mut();
            state.dynamic_state.viewports = Some(vec![viewport]);

            state.framebuffers = new_fbs;
        }
        Ok(())
    }

    /// checks whether the swapchain and/or framebuffers must be
    /// rebuilt. Returns a Result with Ok value representing
    /// state of the recreate swapchain flag
    fn check_swapchain_validity(
        &self,
        window: &Window,
    ) -> Result<bool, failure::Error> {
        let mut recreate_swapchain =
            self.recreate_swapchain.load(Ordering::Acquire);
        let mut rebuild_fbs = false;
        let device = &self.device;
        {
            let mut state = self.state.borrow_mut();
            // state.previous_frame_end.cleanup_finished();

            if recreate_swapchain {
                recreate_swapchain = false;
                rebuild_fbs = true;
                let (width, height) = window.vulkan_drawable_size();
                info!("rebuilding swapchain {}x{}", width, height);
                let (new_swapchain, new_images) = match state
                    .swapchain
                    .recreate_with_dimension([width, height])
                {
                    Ok(r) => r,
                    Err(SwapchainCreationError::UnsupportedDimensions) => {
                        return Err(failure::Error::from(
                            SwapchainCreationError::UnsupportedDimensions,
                        ));
                    }
                    Err(err) => panic!("unexpected error: {:?}", err),
                };
                state.swapchain = new_swapchain;
                state.swapchain_images = new_images;
            }
        }
        if rebuild_fbs {
            if let Err(e) = self.window_size_fb_setup() {
                error!("Problem rebuilding framebuffers");
            }
        }
        Ok(recreate_swapchain)
    }

    fn create_transform_descriptorset(
        &self,
        world: &EntityWorld<Self>,
        modelview: Mat4,
        projection: Mat4,
    ) -> Result<Arc<impl DescriptorSet + Send + Sync>, failure::Error> {
        let ubo_subbuffer = {
            use cgmath::*;
            let data = pipelines::MatrixUniformData::new(modelview, projection)
                .unwrap();
            self.pipelines.matrix_ubo.next(data.into())?
        };

        if let Ok(mut pool) = self.pipelines.matrix_desc_pool.write() {
            pool.next()
                .add_buffer(ubo_subbuffer)
                .map_err(&failure::Error::from)
                .and_then(|pds| pds.build().map_err(&failure::Error::from))
                .map(|pds| Arc::new(pds))
        } else {
            panic!("poisoned RwLock")
        }
    }

    pub fn draw_frame(&self, window: &Window, world: &EntityWorld<Self>) {
        use crate::game::resource::MeshHandle;

        let mut recreate_swapchain = match self.check_swapchain_validity(window)
        {
            Ok(t) => t,
            Err(_) => return,
        };
        let camera_view = world.main_camera.transform();
        {
            let mut prev_frame = self.previous_frame_end.replace(None);
            if let Some(mut fence_fut) = prev_frame {
                fence_fut.cleanup_finished();
                fence_fut.wait(None).unwrap();
            }
        }

        {
            let mut state = self.state.borrow_mut();
            let pipeline = &self.pipelines.main_pipeline;

            let (image_num, acquire_future) =
                match acquire_next_image(state.swapchain.clone(), None) {
                    Ok(r) => r,
                    Err(AcquireError::OutOfDate) => {
                        self.recreate_swapchain.store(true, Ordering::Release);
                        return;
                    }
                    Err(e) => panic!("unexpected error: {:?}", e),
                };

            let clear_values = vec![
                [0.0, 0.0, 1.0, 1.0].into(), // color buffer
                1f32.into(),                 // depth buffer
            ];

            let system = RenderSystem::collect(world);
            
            let VkMesh {
                ref vertex_buffer,
                ref index_buffer,
                ..
            } = &world.resources.meshes[&MeshHandle(0)];

            
            let mut cb_builder: Result<AutoCommandBufferBuilder, failure::Error> =
                AutoCommandBufferBuilder::primary_one_time_submit(
                    self.device.clone(),
                    self.queues.graphics_queue.family(),
                )
                .map_err(&failure::Error::from)
                .and_then(|cb| cb.begin_render_pass(
                     state.framebuffers[image_num].clone(),
                     false,
                     clear_values
                ).map_err(&failure::Error::from));

            for &(entity, mask) in system.entities.iter() {
                let model: Mat4 = system.transforms[entity.0].clone().unwrap().transform.into();
                let modelview = camera_view * model;
                let mesh_handle = system.meshes.get(*entity).map(|c| c.mesh).unwrap();
                let VkMesh{ref index_buffer, ref vertex_buffer, ..} = world.resources.fetch(mesh_handle).unwrap();

                let desc_set = self
                    .create_transform_descriptorset(
                        world,
                        modelview,
                        self.camera().projection,
                    )
                    .expect("could not create descriptor set");
                cb_builder = cb_builder.and_then(|cb|
                    cb.draw_indexed(
                            pipeline.clone(),
                            &state.dynamic_state,
                            vec![vertex_buffer.clone()],
                            index_buffer.clone(),
                            desc_set.clone(),
                            (),
                        )
                        .map_err(&failure::Error::from));
            }
                
            cb_builder = cb_builder.and_then(|cb| {
                    cb.end_render_pass().map_err(&failure::Error::from)
                });
            let command_buffer = cb_builder
                .and_then(|cb| cb.build().map_err(&failure::Error::from))
                .unwrap_or_else(|e| {
                    panic!("could not create command buffer: {:?}", e)
                });

            let future: Box<dyn GpuFuture> = Box::new(
                acquire_future
                    .then_execute(
                        self.queues.graphics_queue.clone(),
                        command_buffer,
                    )
                    .unwrap()
                    .then_swapchain_present(
                        self.queues.graphics_queue.clone(),
                        state.swapchain.clone(),
                        image_num,
                    ),
            );

            match future.then_signal_fence_and_flush() {
                Ok(f) => {
                    self.previous_frame_end.replace(Some(f));
                }
                Err(FlushError::OutOfDate) => {
                    self.recreate_swapchain.store(true, Ordering::Release);
                }
                Err(e) => {
                    error!("vulkan error: {:?}", e);
                }
            }
        }

        self.recreate_swapchain
            .store(recreate_swapchain, Ordering::Release);
    }
}

#[derive(Debug, Clone)]
pub struct VkTexture;

impl Renderer for VulkanRenderer {
    type Texture = VkTexture;
    type Mesh = VkMesh;
    fn camera(&self) -> Ref<Camera> {
        self.camera.borrow()
    }

    fn on_resize(&self, _size: (u32, u32)) {
        self.recreate_swapchain.store(true, Ordering::Relaxed);
    }
}


struct RenderSystem<'a> {
    entities: Vec<(Entity, ComponentMask)>,
    meshes: &'a IndexArray<MeshComponent>,
    transforms: &'a IndexArray<TransformComponent>,
}


impl<'a> RenderSystem<'a> {

    fn collect(world: &'a EntityWorld<VulkanRenderer>) -> Self {
        let required_mask: ComponentMask = ComponentMask::MESH | ComponentMask::TRANSFORM;

        RenderSystem {
            entities: world.components.entities().
                flat_map(|e| {
                    let mask = world.components.masks[*e].unwrap_or(ComponentMask::NONE);
                    if mask.contains(required_mask){
                        Some((e, mask))
                    }  else {
                        None
                    }
                }).collect(),
            meshes: &world.components.meshes,
            transforms: &world.components.transforms
        }
    }
}