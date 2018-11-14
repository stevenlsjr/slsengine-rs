// use super::ash;

pub use self::sdl_vulkan::prelude::*;
use super::{get_error_desc, AppError};

use failure;
use renderer_common::*;
use sdl2;
use sdl2::video::Window as SdlWindow;

use std::cell::{Ref, RefCell};
use std::default::Default;
use std::ffi::{CStr, CString};
use std::sync::Arc;
use std::{mem, ptr};

use vulkano;
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::swapchain::Surface;

pub mod sdl_vulkan;

type SdlSurface = Surface<SdlWindow>;
static MAIN_QUEUE_PRIORITY: f32 = 1.0;

/// Error Record for VkContext creation
#[derive(Fail, Clone, Debug)]
pub enum VkContextError {
    #[fail(display = "could not create entry")]
    Entry,
    #[fail(display = "could not create instance")]
    Instance,
    #[fail(display = "could not load {} extensions", _0)]
    Extensions(String),
    #[fail(display = "could not load Vulkan SurfaceKHR functions")]
    SurfaceLoader,
    #[fail(display = "could not find Queue Families")]
    QueueFamilies,
    #[fail(display = "could not create surfaceKHR")]
    Surface,
    #[fail(display = "could not create device")]
    Device,

    #[fail(
        display = "Could not create vulkan rendering context: {:#?}",
        _0
    )]
    Other(String),
}

fn rate_physical_device(phys_dev: PhysicalDevice) -> i32 {
    use vulkano::instance::PhysicalDeviceType;
    let mut score = 0;
    let dev_features = phys_dev.supported_features();

    for &feature in [dev_features.geometry_shader].iter() {
        if feature {
            score += 100
        }
    }

    match phys_dev.ty() {
        PhysicalDeviceType::DiscreteGpu => score += 1000,
        PhysicalDeviceType::IntegratedGpu => score += 500,
        _ => {}
    }
    score
}

pub fn pick_physical_device(
    instance: &Arc<Instance>,
) -> Result<PhysicalDevice, failure::Error> {
    let mut top_device = (0, None::<PhysicalDevice>);
    for dev in PhysicalDevice::enumerate(instance) {
        let rating = rate_physical_device(dev);
        if top_device.1.is_none() || rating > top_device.0 {
            top_device = (rating, Some(dev));
        }

        println!("device {:?} {:?} with rating {}", dev, dev.name(), rating);
    }
    match top_device.1 {
        Some(dev) => Ok(dev),
        None => Err(format_err!("could not find a suitible physical device")),
    }
}
