// use super::ash;

use ash::extensions::{DebugReport, Surface};

#[cfg(target_os = "macos")]
use ash::extensions::MacOSSurface;
#[cfg(target_os = "windows")]
use ash::extensions::Win32Surface;
#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
use ash::extensions::XlibSurface;

use super::{get_error_desc, AppError};
use ash::version::{EntryV1_0, InstanceV1_0, V1_0};
use ash::vk;
use ash::vk::types as vkt;
use ash::vk::types::{
    PhysicalDevice, PhysicalDeviceFeatures, PhysicalDeviceProperties,
};
use ash::{Entry, Instance};
use std::default::Default;
use std::ffi::CStr;

use renderer_common::*;
use std::cell::{Ref, RefCell};

/// from Ash example,
#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
pub fn get_ext_names() -> Vec<*const i8> {
    vec![
        Surface::name().as_ptr() as *const i8,
        XlibSurface::name().as_ptr() as *const i8,
        DebugReport::name().as_ptr() as *const i8,
    ]
}

#[cfg(target_os = "macos")]
pub fn get_ext_names() -> Vec<*const i8> {
    vec![
        Surface::name().as_ptr() as *const i8,
        MacOSSurface::name().as_ptr() as *const i8,
        DebugReport::name().as_ptr() as *const i8,
    ]
}

#[cfg(all(windows))]
pub fn get_ext_names() -> Vec<*const i8> {
    vec![
        Surface::name().as_ptr() as *const i8,
        Win32Surface::name().as_ptr() as *const i8,
        DebugReport::name().as_ptr() as *const i8,
    ]
}

/// provides app defaults for vkInstance creation
pub fn make_instance(entry: &Entry<V1_0>) -> Result<Instance<V1_0>, String> {
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
        .map_err(get_error_desc);
    instance
}

/// rates device suitibility. Ported from
/// https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Physical_devices_and_queue_families#page_Base_device_suitability_checks
fn rate_device(
    properties: &PhysicalDeviceProperties,
    _features: &PhysicalDeviceFeatures,
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

fn get_device_name(props: &PhysicalDeviceProperties) -> &str {
    let cs = unsafe { CStr::from_ptr(&props.device_name as *const _) };
    for (c, i) in cs.to_bytes().iter().zip(0..) {
        if *c == 0 {
            break;
        }
        if i >= vkt::VK_MAX_PHYSICAL_DEVICE_NAME_SIZE {
            return "invalid name";
        }
    }

    cs.to_str().unwrap_or("invalid name")
}

pub fn pick_physical_device(
    instance: &Instance<V1_0>,
) -> Result<vkt::PhysicalDevice, AppError> {
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
        let _device_name = get_device_name(&properties);
        if score >= top_device.0 {
            top_device = (score, Some(*device));
        }
    }
    let chosen_device =
        top_device.1.expect("If no devices availible, pick_physical_device should have already returned");
    let _queue_family =
        find_queue_family(instance, &chosen_device).ok_or(AppError::Misc(
            format!("could not find suitible queue family for device"),
        ))?;

    Ok(chosen_device)
}

pub fn find_queue_family(
    instance: &Instance<V1_0>,
    device: &PhysicalDevice,
) -> Option<QueueFamilyIndex> {
    let queue_families =
        instance.get_physical_device_queue_family_properties(*device);
    let mut index = 0;

    for queue_family in queue_families {
        let mask = vk::QUEUE_GRAPHICS_BIT & vk::QUEUE_COMPUTE_BIT;
        let has_queue_count = queue_family.queue_count > 0;
        let is_valid =
            has_queue_count && (queue_family.queue_flags & mask) == mask;
        println!(
            "mask:0x{:x}, has_queue_count:{}, is_valid:{}",
            mask.flags(),
            has_queue_count,
            is_valid
        );
        if is_valid {
            return Some(QueueFamilyIndex {
                properties: queue_family,
                index,
            });
        }
        index += 1;
    }
    None
}

#[derive(Debug, Clone)]
pub struct QueueFamilyIndex {
    pub index: u32,
    pub properties: vkt::QueueFamilyProperties,
}

pub fn create_logical_device(
    instance: &Instance<V1_0>,
    physical_device: &PhysicalDevice,
) -> Result<(), AppError> {
    // use std::ptr;
    let _queue_index = find_queue_family(instance, physical_device)
        .ok_or(AppError::Misc("could not find graphics family".to_string()))?;
    // let queue_create_info: vk::DeviceQueueCreateInfo = vk::DeviceQueueCreateInfo {
    //     s_type: vk::StructureType::DeviceCreateInfo,
    //     p_next: ptr::null(),
    //     flags: Default::default(),
    //     queue_create_info_count: 0,
    //     p_queue_create_infos: ptr::null(),
    //     enabled_layer_count: 0,
    //     pp_enabled_layer_names: ptr::null(),
    //     enabled_extension_count: 0,
    //     pp_enabled_extension_names: ptr::null(),
    //     p_enabled_features: ptr::null(),
    // };

    Ok(())
}

#[cfg(test)]
fn stub_physical_device() -> PhysicalDeviceProperties {
    use ash::vk::types::*;
    use std::mem::uninitialized;

    unsafe {
        PhysicalDeviceProperties {
            api_version: 0,
            driver_version: 0,
            vendor_id: 0,
            device_id: 0,
            device_type: PhysicalDeviceType::DiscreteGpu,
            device_name: [0i8; VK_MAX_PHYSICAL_DEVICE_NAME_SIZE],
            pipeline_cache_uuid: [0u8; VK_UUID_SIZE],
            limits: uninitialized(),
            sparse_properties: uninitialized(),
        }
    }
}
#[test]
fn test_device_name() {
    let mut dev_properties = stub_physical_device();
    let new_name =
        vec!['h' as i8, 'e' as i8, 'l' as i8, 'l' as i8, 'o' as i8, 0i8];
    {
        dev_properties.device_name[0..new_name.len()]
            .copy_from_slice(new_name.as_slice());
        let cs = get_device_name(&dev_properties);
        assert_eq!(cs, "hello");
    }
    {
        dev_properties
            .device_name
            .copy_from_slice(&[10i8; vkt::VK_MAX_PHYSICAL_DEVICE_NAME_SIZE]);
        assert_eq!(get_device_name(&dev_properties), "invalid name");
    }
}

pub struct VkRenderer {
    camera: RefCell<Camera>
}

impl VkRenderer {

    pub fn new() -> VkRenderer {
        let camera = Camera::new(default_perspective());
        VkRenderer {
            camera: RefCell::new(camera)
        }
    }
}


impl Renderer for VkRenderer {
    fn clear(&self) {
    }

    fn camera(&self) -> Ref<Camera> {
        self.camera.borrow()
    }

    fn set_clear_color(&mut self, color: Color) {
        unimplemented!()
    }
}
