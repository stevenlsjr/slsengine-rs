#![allow(unused_imports)]
#![allow(unused_variables)]
#![cfg(feature = "with-vulkan")]

extern crate ash;
#[macro_use]
extern crate failure;
extern crate sdl2;
extern crate slsengine;

use ash::version::*;
use ash::vk::types as vkt;
use ash::vk::PhysicalDevice;
use ash::Entry;
use ash::*;
use sdl2::sys as sdl_sys;
use sdl2::video::*;
use sdl2::*;
use slsengine::renderer_vk::*;
use slsengine::sdl_platform::*;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

pub type AppEntry = Entry<V1_0>;

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

fn main() {
    use std::mem;
    use std::thread;
    use std::time::Duration;
    let platform = platform().build(&VulkanPlatformHooks).unwrap();

    let context =
        VkContext::new(&platform.window).expect("could not create context");

    // let renderer = VkRenderer::new();
}
