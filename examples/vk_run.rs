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

pub struct VkContext {
    pub instance: Instance<V1_0>,
    pub entry: Entry<V1_0>,
    pub surface_ext: extensions::Surface,
    pub queue_families: QueueFamilies,
    pub surface: vk::SurfaceKHR
}



static MAIN_QUEUE_PRIORITY: f32 = 1.0;

fn main() {
    use std::mem;
    use std::thread;
    use std::time::Duration;
    let platform = platform().build(&VulkanPlatformHooks).unwrap();

    let enable_validation_layers = true;
    let validation_layers: Vec<CString> =
        vec![CString::new("VK_LAYER_LUNARG_standard_validation").unwrap()];
    let validation_layer_ptrs: Vec<*const i8> =
        validation_layers.iter().map(|name| name.as_ptr()).collect();

    let entry = Entry::new().unwrap();
    let instance: Instance<V1_0> =
        make_instance(&entry, &validation_layers, &platform.window).unwrap();

    let surface = platform.window.create_vk_surface(&instance).unwrap();
    let phys_dev = pick_physical_device(&instance)
        .expect("Couldn't create physical device");

    let surface_ext = extensions::Surface::new(&entry, &instance).unwrap();

    let surface_capabilities = surface_ext
        .get_physical_device_surface_capabilities_khr(phys_dev, surface)
        .unwrap();

    let queue_families = QueueFamilies::new(&instance, &phys_dev, &surface_ext, surface)
        .unwrap();



    let device = make_device(&instance, phys_dev, &queue_families.make_create_info_vec(), &validation_layers).unwrap();
    {
        let ctx = VkContext{
            instance,
            entry,
            surface_ext,
            queue_families,
            surface,
        };
    }



    // let renderer = VkRenderer::new();
}
