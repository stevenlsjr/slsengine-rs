#![allow(unused_imports)]
#![allow(unused_variables)]
#![cfg(feature = "with-vulkan")]

extern crate ash;

#[macro_use]
extern crate failure;
extern crate sdl2;
extern crate slsengine;
use ash::version::*;
use ash::vk::PhysicalDevice;

use ash::vk::types as vkt;
use ash::Entry;
use ash::*;
use sdl2::video::*;
use sdl2::*;
use slsengine::renderer_vk::*;
use slsengine::sdl_platform::*;

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

use sdl2::sys as sdl_sys;
use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use std::ptr;

static MAIN_QUEUE_PRIORITY: f32 = 1.0;


fn print_sdl_extensions(
    window: &sdl2::video::Window,
) -> Result<Vec<CString>, failure::Error> {
    use slsengine::renderer_vk::sdl_vulkan::*;
    use sdl_sys::SDL_bool;
    let mut n_extensions: u32 = 0;
    let p_window = window.raw();
    
    let mut cstrings: Vec<CString> = Vec::new();
    let names: Vec<*const i8> = window.vk_instance_extensions().unwrap();
    for p_ext_name in names  {
        let name = unsafe { CStr::from_ptr(p_ext_name)};
        let owned = CString::new(name.to_bytes()).unwrap();
        cstrings.push(owned);
        println!("exension {}, {:#?}", name.to_str().unwrap_or("!invalid name"), name);
    }
    Ok(cstrings)
}

fn main() {
    use std::mem;
    use std::thread;
    use std::time::Duration;
    let platform = platform().build(&VulkanPlatformHooks).unwrap();
    print_sdl_extensions(&platform.window).unwrap();

    let enable_validation_layers = true;
    let validation_layers: Vec<CString> =
        vec![CString::new("VK_LAYER_LUNARG_standard_validation").unwrap()];
    let validation_layer_ptrs: Vec<*const i8> =
        validation_layers.iter().map(|name| name.as_ptr()).collect();

    let entry = Entry::new().unwrap();
    let instance: Instance<V1_0> =
        make_instance(&entry, &validation_layers, &platform.window).unwrap();
    let phys_dev = pick_physical_device(&instance)
        .expect("Couldn't create physical device");

    let QueueFamilies{graphics_family, present_family} = QueueFamilies::new(&instance, &phys_dev).unwrap();
    let queue_create_info = vk::DeviceQueueCreateInfo {
        s_type: vk::StructureType::DeviceQueueCreateInfo,
        queue_family_index: graphics_family.index,
        queue_count: 1,
        p_queue_priorities: &MAIN_QUEUE_PRIORITY as *const _,
        p_next: ptr::null(),
        flags: Default::default(),
    };

    let device: ash::Device<V1_0> = {
        let dev_features: vk::PhysicalDeviceFeatures = unsafe { mem::zeroed() };
        let mut create_info: vk::DeviceCreateInfo = unsafe { mem::zeroed() };
        create_info.s_type = vk::StructureType::DeviceCreateInfo;
        create_info.enabled_extension_count = 0;

        create_info.p_queue_create_infos = &queue_create_info;
        create_info.queue_create_info_count = 1;
        if enable_validation_layers {
            create_info.enabled_layer_count = validation_layers.len() as u32;
            create_info.pp_enabled_layer_names = validation_layer_ptrs.as_ptr();
        } else {
            create_info.enabled_extension_count = 0;
        }

        unsafe {
            instance
                .create_device(phys_dev, &create_info, None)
                .unwrap()
        }
    };

    let graphics_queue: vkt::Queue =
        unsafe { device.get_device_queue(graphics_family.index, 0) };

    println!(
        "created device {:?}, graphics queue {:?}",
        device.handle(),
        graphics_queue
    );

    // let device = create_logical_device(&instance, &phys_dev).unwrap();
    // let renderer = VkRenderer::new();
}
