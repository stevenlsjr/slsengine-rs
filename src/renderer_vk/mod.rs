// use super::ash;

use ash;
use ash::extensions::{DebugReport, Surface};
use ash::version::*;
use ash::version::{EntryV1_0, InstanceV1_0, V1_0};
use ash::vk;
use ash::vk::types as vkt;
use ash::vk::types::{
    PhysicalDevice, PhysicalDeviceFeatures, PhysicalDeviceProperties,
};
use ash::{Entry, Instance};
use failure;
use renderer_common::*;
use sdl2;
use sdl2::video::Window as SdlWindow;
// enable sdl_vulkan extension traits
pub use self::sdl_vulkan::prelude::*;
use super::{get_error_desc, AppError};
use std::cell::{Ref, RefCell};
use std::default::Default;
use std::ffi::CStr;
use std::ffi::CString;
use std::{mem, ptr};

pub mod sdl_vulkan;

static MAIN_QUEUE_PRIORITY: f32 = 1.0;

pub fn make_instance(
    entry: &Entry<V1_0>,
    validation_layers: &[CString],
    window: &sdl2::video::Window,
) -> Result<Instance<V1_0>, failure::Error> {
    use self::sdl_vulkan::prelude::*;
    use std::ptr;

    let app_name = CString::new("Hello vulkan").unwrap();

    let layer_name_ptrs: Vec<*const i8> =
        validation_layers.iter().map(|name| name.as_ptr()).collect();
    let mut ext_names = window.vk_instance_extensions()?;
    ext_names.push(DebugReport::name().as_ptr() as *const _);

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
        .map_err(|e| failure::err_msg(get_error_desc(e)));
    instance
}

pub fn make_device(
    instance: &Instance<V1_0>,
    phys_dev: vk::PhysicalDevice,
    queue_create_info: &[vk::DeviceQueueCreateInfo],
    validation_layers: &[CString],
) -> Result<ash::Device<V1_0>, ()> {
    let validation_layer_ptrs: Vec<*const i8> =
        validation_layers.iter().map(|name| name.as_ptr()).collect();
    let dev_features: vk::PhysicalDeviceFeatures = unsafe { mem::zeroed() };
    let mut create_info: vk::DeviceCreateInfo = unsafe { mem::zeroed() };
    create_info.s_type = vk::StructureType::DeviceCreateInfo;
    create_info.enabled_extension_count = 0;

    create_info.p_queue_create_infos = queue_create_info.as_ptr();
    create_info.queue_create_info_count = queue_create_info.len() as u32;
    create_info.enabled_layer_count = validation_layers.len() as u32;
    create_info.pp_enabled_layer_names = validation_layer_ptrs.as_ptr();

    unsafe {
        Ok(instance
            .create_device(phys_dev, &create_info, None)
            .unwrap())
    }
}

/// rates device suitibility. Ported from
/// https://vulkan-tutorial.com/Drawing_a_triangle
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
) -> Result<vkt::PhysicalDevice, failure::Error> {
    let physical_devices = instance
        .enumerate_physical_devices()
        .map_err(|e| format_err!("could not fetch physical devices"))?;
    if physical_devices.len() == 0 {
        bail!("No physical devices found")
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

    Ok(chosen_device)
}

/// Stores vulkan queue family index along with family properties.
#[derive(Debug, Clone)]
pub struct QueueFamilyIndex {
    pub index: u32,
    pub properties: vkt::QueueFamilyProperties,
}

#[derive(Debug, Clone)]
pub struct QueueFamilies {
    pub graphics_family: QueueFamilyIndex,
    pub present_family: QueueFamilyIndex,
}

impl QueueFamilies {
    pub fn new(
        instance: &Instance<V1_0>,
        device: &PhysicalDevice,
        surface_ext: &ash::extensions::Surface,
        surface: vkt::SurfaceKHR,
    ) -> Result<QueueFamilies, failure::Error> {
        use ash::extensions::Surface;

        let mut graphics_family: Option<QueueFamilyIndex> = None;
        let mut present_family: Option<QueueFamilyIndex> = None;
        let queue_families =
            instance.get_physical_device_queue_family_properties(*device);

        for index in 0..queue_families.len() {
            let queue_family = queue_families[index].clone();
            if queue_family.queue_count <= 0 {
                continue;
            }

            let mask = vk::QUEUE_GRAPHICS_BIT & vk::QUEUE_COMPUTE_BIT;
            let is_graphics_queue = (queue_family.queue_flags & mask) == mask;

            let is_present_queue = surface_ext
                .get_physical_device_surface_support_khr(
                    *device,
                    index as u32,
                    surface,
                );
            if is_graphics_queue {
                graphics_family = Some(QueueFamilyIndex {
                    properties: queue_family.clone(),
                    index: index as u32,
                });
            }

            if is_present_queue {
                present_family = Some(QueueFamilyIndex {
                    properties: queue_family,
                    index: index as u32,
                });
            }

            let is_complete =
                graphics_family.is_some() & &present_family.is_some();

            if is_complete {
                return Ok(QueueFamilies {
                    graphics_family: graphics_family.unwrap(),
                    present_family: present_family.unwrap(),
                });
            }
        }
        bail!("Required queue families not found")
    }

    pub fn make_create_info_vec(&self) -> Vec<vk::DeviceQueueCreateInfo> {
        use std::collections::HashSet;
        use std::iter::FromIterator;
        let mut infos: Vec<vk::DeviceQueueCreateInfo> = Vec::new();
        let mut unique_ids: HashSet<u32, _> = HashSet::new();
        unique_ids.insert(self.graphics_family.index);
        unique_ids.insert(self.present_family.index);


        for index in unique_ids.iter().map(|i|*i) {
            let create_info = vk::DeviceQueueCreateInfo {
                s_type: vk::StructureType::DeviceQueueCreateInfo,
                p_next: ptr::null(),
                flags: Default::default(),
                queue_family_index: index,
                queue_count: 1,
                p_queue_priorities: &MAIN_QUEUE_PRIORITY,
            };
            infos.push(create_info);

        }

        infos
    }

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
    camera: RefCell<Camera>,
}

impl VkRenderer {
    pub fn new() -> VkRenderer {
        let camera = Camera::new(default_perspective());
        VkRenderer {
            camera: RefCell::new(camera),
        }
    }
}

impl Renderer for VkRenderer {
    fn clear(&self) {}

    fn camera(&self) -> Ref<Camera> {
        self.camera.borrow()
    }

    fn set_clear_color(&mut self, _color: Color) {
        unimplemented!()
    }
}
