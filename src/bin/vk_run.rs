#[macro_use]
extern crate ash;
#[macro_use]
extern crate failure;
extern crate slsengine;

use ash::prelude::VkResult;
use ash::version::{EntryV1_0, InstanceV1_0, V1_0};
use ash::vk;
use ash::vk::types as vkt;
use ash::vk::types::{PhysicalDeviceFeatures, PhysicalDeviceProperties};
use ash::Device;
use ash::Entry;
use ash::Instance;
use slsengine::renderer_vk::*;
use slsengine::sdl_platform::*;
use std::default::Default;
use std::ffi::{CStr, CString};
use std::rc::Rc;

struct VulkanPlatformHooks;

#[derive(Fail, Debug)]
enum AppError {
    #[fail(display = "vulkan error: {}", _0)]
    VkError(vk::Result),

    #[fail(display = "misc error: '{}'", _0)]
    Misc(String),
}

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

///
fn make_instance(entry: &Entry<V1_0>) -> Result<Instance<V1_0>, String> {
    use std::ffi::CString;
    use std::ptr;

    let app_name = CString::new("Hello vulkan").unwrap();
    let layer_names =
        [CString::new("VK_LAYER_LUNARG_standard_validation").unwrap()];
    let layer_name_ptrs: Vec<*const i8> =
        layer_names.iter().map(|name| name.as_ptr()).collect();
    let ext_names = get_ext_names();

    let app_info = vk::ApplicationInfo {
        s_type: vk::StructureType::ApplicationInfo,
        p_next: ptr::null(),
        p_application_name: app_name.as_ptr(),
        application_version: 0,
        p_engine_name: app_name.as_ptr(),
        engine_version: 0,
        api_version: vk_make_version!(1, 0, 36),
    };
    let create_info = vk::InstanceCreateInfo {
        s_type: vk::StructureType::InstanceCreateInfo,
        p_next: ptr::null(),
        flags: Default::default(),
        p_application_info: &app_info,
        pp_enabled_layer_names: layer_name_ptrs.as_ptr(),
        enabled_layer_count: layer_name_ptrs.len() as u32,
        pp_enabled_extension_names: ext_names.as_ptr(),
        enabled_extension_count: ext_names.len() as u32,
    };

    let instance = unsafe { entry.create_instance(&create_info, None) }
        .map_err(slsengine::get_error_desc);
    instance
}

/// rates device suitibility. Ported from
/// https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Physical_devices_and_queue_families#page_Base_device_suitability_checks
fn rate_device(
    properties: &PhysicalDeviceProperties,
    features: &PhysicalDeviceFeatures,
) -> i32 {
    let mut score: i32 = 0;
    match properties.device_type {
        vk::PhysicalDeviceType::DiscreteGpu => score += 1000,
        vk::PhysicalDeviceType::IntegratedGpu => score += 500,
        _ => {}
    };

    score += properties.limits.max_image_dimension2d as i32;
    //
    //    if features.geometry_shader == 0 {
    //        return 0;
    //    }
    score
}

fn pick_physical_device(instance: &Instance<V1_0>) -> Result<vkt::PhysicalDevice, AppError> {
    let physical_devices = instance
        .enumerate_physical_devices()
        .map_err(AppError::VkError)?;
    if physical_devices.len() == 0 {
        return Err(AppError::Misc("No physical devices found".to_string()));
    }

    let mut top_device = (0, None);

    for device in physical_devices.iter() {
        let properties: PhysicalDeviceProperties =
            instance.get_physical_device_properties(*device);
        let features: PhysicalDeviceFeatures =
            instance.get_physical_device_features(*device);
        let score = rate_device(&properties, &features);
        use std::str;
        let device_name =
            unsafe { CStr::from_ptr(&properties.device_name as *const _) }
                .to_str()
                .unwrap_or("unknown name");
        if score >= top_device.0 {
            top_device = (score, Some(*device));
        }
    }
    let chosen_device = top_device.1.expect("A suitible device should be inserted");
    Ok(chosen_device)
}

fn main() {
    use std::ptr;
    use std::thread;
    use std::time::Duration;
    let platform = platform().build(&VulkanPlatformHooks).unwrap();
    let entry = Entry::new().unwrap();

    let instance = make_instance(&entry).unwrap();
    pick_physical_device(&instance).expect("Something's not right");

    //    let mut main_loop = slsengine::MainLoopState::new();
    //    main_loop.is_running = true;
    //    while main_loop.is_running {
    //        main_loop.handle_events(
    //            &platform.window,
    //            platform.event_pump.borrow_mut().poll_iter(),
    //        );
    //        thread::sleep(Duration::from_millis(16));
    //    }
}
