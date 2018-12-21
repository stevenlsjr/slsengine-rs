#![allow(unused_imports)]
#![allow(unused_variables)]
#![cfg(feature = "with-vulkan")]

#[macro_use]
extern crate failure;

use cgmath::prelude::*;
use sdl2::sys as sdl_sys;
use sdl2::video::*;
use sdl2::*;
use slsengine::renderer::backend_vk::*;
use slsengine::renderer::Camera;
use slsengine::sdl_platform::*;
use std::cell::{Ref, RefCell};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use std::rc::Rc;
use std::sync::Arc;
use vulkano::{self, impl_vertex};
use vulkano_shaders;

mod vs {
    vulkano_shaders::shader! {
    ty: "vertex",
        src: "
        #version 450

        layout(location = 0) in vec3 position;
        
        void main(){
            gl_Position = vec4(position, 1.0);
        }
        "
    }
}

mod fs {
    vulkano_shaders::shader! {
    ty: "fragment",
        src: "
        #version 450

        layout(location = 0) out vec4 out_color;
        
        void main(){
            out_color = vec4(1.0, 1.0, 1.0, 1.0);
        }
        "
    }

}

struct VulkanPlatformHooks;

// static FRAG_SPIRV: &[u8] =
//     include_bytes!("../assets/shaders/spirv/brdf.frag.spv");

// static VERT_SPIRV: &[u8] =
//     include_bytes!("../assets/shaders/spirv/brdf.vert.spv");

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

fn main() {
    use slsengine::game;
    use std::time::{Duration, Instant};
    use vulkano::pipeline::*;

    let platform = platform().build(&VulkanPlatformHooks).unwrap();

    let mut loop_state = slsengine::MainLoopState::new();
    let timer = game::Timer::new(Duration::from_millis(100 / 6));

    let renderer = VulkanRenderer::new(&platform.window).unwrap();
    let fs = fs::Shader::load(renderer.device.clone()).unwrap();
    let vs = vs::Shader::load(renderer.device.clone()).unwrap();

    loop_state.is_running = true;
}
