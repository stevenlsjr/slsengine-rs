#![allow(unused_imports)]
#![allow(unused_variables)]
#![cfg(feature = "with-vulkan")]

#[macro_use]
extern crate failure;
extern crate sdl2;
extern crate slsengine;

#[macro_use]
extern crate vulkano;

use sdl2::sys as sdl_sys;
use sdl2::video::*;
use sdl2::*;
use slsengine::sdl_platform::*;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

use slsengine::renderer::Camera;
use slsengine::renderer_vk::*;
use std::cell::{Ref, RefCell};

struct VulkanPlatformHooks;

static FRAG_SPIRV: &[u8] = include_bytes!("../assets/vulkan/flat-shading.frag.spv");
static VERT_SPIRV: &[u8] = include_bytes!("../assets/vulkan/flat-shading.vert.spv");

impl PlatformBuilderHooks for VulkanPlatformHooks {
    fn build_window(
        &self,
        platform_builder: &PlatformBuilder,
        video_subsystem: &VideoSubsystem,
    ) -> PlatformResult<Window> {
        let mut wb = make_window_builder(platform_builder, video_subsystem);
        wb.vulkan();
        wb.resizable();
        let window = wb.build().unwrap();
        Ok(window)
    }
}
extern crate cgmath;
use cgmath::prelude::*;
use std::sync::Arc;
use vulkano::instance::Instance;
use vulkano::swapchain::Surface;
use std::rc::Rc;
use vulkano::device::Device;

#[derive(Debug)]
pub struct VulkanRenderer {
    camera: RefCell<Camera>,
    instance: Arc<Instance>,
    device: Arc<Device>,
    queues: VulkanQueues,
    surface: Arc<Surface<Rc<WindowContext>>>

}
impl VulkanRenderer {
    pub fn new(window: &Window) -> Result<Self, VkContextError> {
        use vulkano::instance::{Instance, PhysicalDevice, RawInstanceExtensions};
        use vulkano::swapchain::Surface;
        use vulkano::VulkanObject;
        let instance_extensions =
            window.vulkan_instance_extensions().unwrap();
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
        let physical_device = pick_physical_device(&instance).unwrap();
        let (device, queues) = create_device(&physical_device, &surface).unwrap();

        Ok(VulkanRenderer {
            camera: RefCell::new(Camera::new(cgmath::PerspectiveFov {
                fovy: cgmath::Deg(40.0).into(),
                aspect: 1.0,
                near: 0.1,
                far: 100.0,
            })),
            instance,
            device,
            queues,
            surface,
        })
    }
}

impl slsengine::renderer::Renderer for VulkanRenderer {
    fn camera(&self) -> Ref<Camera> {
        self.camera.borrow()
    }
}

fn main() {
    use slsengine::game;
    use std::time::{Duration, Instant};

    let platform = platform().build(&VulkanPlatformHooks).unwrap();



    //    let mut loop_state = slsengine::MainLoopState::new();
    //    let mut timer = game::Timer::new(Duration::from_millis(100 / 6));
    //    let mut renderer = VulkanRenderer::new();
    //    loop_state.is_running = true;
    //    while loop_state.is_running {
    //        loop_state.handle_events(
    //            &platform.window,
    //            platform.event_pump.borrow_mut().poll_iter(),
    //            &mut renderer,
    //        );
    //        let game::Tick { delta: _delta, .. } = timer.tick();
    //
    //        let ticks = Instant::now().duration_since(timer.start_instant());
    //        let theta = game::duration_as_f64(ticks);
    //
    //
    //
    //    }
}
