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

static FRAG_SPIRV: &[u8] =
    include_bytes!("../assets/shaders/spirv/flat-shading.frag.spv");

static VERT_SPIRV: &[u8] =
    include_bytes!("../assets/shaders/spirv/flat-shading.vert.spv");

static COMPUTE_SPIRV: &[u8] =
    include_bytes!("../assets/shaders/spirv/sample_compute.comp.spv");

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
use std::rc::Rc;
use std::sync::Arc;
use vulkano::device::Device;
use vulkano::instance::Instance;
use vulkano::swapchain::Surface;

fn main() {
    use slsengine::game;
    use std::time::{Duration, Instant};
    use vulkano::pipeline::*;

    let platform = platform().build(&VulkanPlatformHooks).unwrap();

    let mut loop_state = slsengine::MainLoopState::new();
    let mut timer = game::Timer::new(Duration::from_millis(100 / 6));

    let renderer = VulkanRenderer::new(&platform.window).unwrap();
    let device = renderer.device.clone();

    loop_state.is_running = true;
}
