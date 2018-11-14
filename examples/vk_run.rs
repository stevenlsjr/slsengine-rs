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
struct VulkanRenderer {
    camera: RefCell<Camera>,
}
impl VulkanRenderer {
    fn new() -> Self {
        VulkanRenderer {
            camera: RefCell::new(Camera::new(cgmath::PerspectiveFov {
                fovy: cgmath::Deg(40.0).into(),
                aspect: 1.0,
                near: 0.1,
                far: 100.0,
            })),
        }
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
    use vulkano::instance::{Instance, RawInstanceExtensions, PhysicalDevice};
    use vulkano::swapchain::Surface;
    use vulkano::VulkanObject;
    let platform = platform().build(&VulkanPlatformHooks).unwrap();

    let instance_extensions =
        platform.window.vulkan_instance_extensions().unwrap();
    let raw_instance_extensions = RawInstanceExtensions::new(
        instance_extensions
            .iter()
            .map(|&v| CString::new(v).unwrap()),
    );
    let layers = vec!["VK_LAYER_LUNARG_standard_validation"];
    let instance = Instance::new(None, raw_instance_extensions, layers)
        .expect("failed to create vulkan instance");
    let surface = {
        let handle = platform
            .window
            .vulkan_create_surface(instance.internal_object())
            .unwrap();
        unsafe {
            Surface::from_raw_surface(
                instance.clone(),
                handle,
                platform.window.context(),
            )
        }
    };
    let physical_device = pick_physical_device(&instance).unwrap();

    let mut loop_state = slsengine::MainLoopState::new();
    let mut timer = game::Timer::new(Duration::from_millis(100 / 6));
    let mut renderer = VulkanRenderer::new();
    loop_state.is_running = true;
    while loop_state.is_running {
        loop_state.handle_events(
            &platform.window,
            platform.event_pump.borrow_mut().poll_iter(),
            &mut renderer,
        );
        let game::Tick { delta: _delta, .. } = timer.tick();

        let ticks = Instant::now().duration_since(timer.start_instant());
        let theta = game::duration_as_f64(ticks);



    }
}
