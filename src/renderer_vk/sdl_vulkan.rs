/**
 * bindings for SDL2's vulkan interface
 */
use failure;
use sdl2::sys::{SDL_Window, SDL_bool};
use sdl2::version::version;
use sdl2::video::Window;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::{mem, ptr};

use ash;
use ash::version::*;
use ash::vk::types as vkt;

// #[cfg(target_os = "macos")]
// use ash::extensions::MacOSSurface;
// #[cfg(target_os = "windows")]
// use ash::extensions::Win32Surface;
// #[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
// use ash::extensions::XlibSurface;

extern "C" {
    pub fn SDL_Vulkan_GetInstanceExtensions(
        window: *mut SDL_Window,
        pCount: *mut u32,
        pNames: *mut *const c_char,
    ) -> SDL_bool;

    pub fn SDL_Vulkan_CreateSurface(
        window: *mut SDL_Window,
        instance: vkt::Instance,
        surface: *mut vkt::SurfaceKHR,
    ) -> SDL_bool;
}

///
/// Trait declarations for safe import of sdl_vulkan
pub mod prelude {
    pub use super::GetInstanceExtensions;
}

pub trait GetInstanceExtensions {
    fn vk_instance_extensions(
        &self,
    ) -> Result<Vec<*const i8>, ::failure::Error>;
    fn vk_instance_extensions_ctsrs(
        &self,
    ) -> Result<Vec<&'static CStr>, ::failure::Error> {
        let pointers: Vec<*const i8> = self.vk_instance_extensions()?;
        let mut cstrs: Vec<&'static CStr> = Vec::with_capacity(pointers.len());
        for p in pointers {
            let cstring = unsafe { CStr::from_ptr(p) };
            cstrs.push(cstring);
        }
        Ok(cstrs)
    }

    fn create_vk_surface(
        &self,
        instance: &ash::Instance<V1_0>,
    ) -> Result<vkt::SurfaceKHR, failure::Error>;
}

impl GetInstanceExtensions for Window {
    fn vk_instance_extensions(&self) -> Result<Vec<*const i8>, failure::Error> {
        let mut n_names: u32 = 0;
        unsafe {
            if SDL_Vulkan_GetInstanceExtensions(
                self.raw(),
                &mut n_names,
                ptr::null_mut(),
            ) != SDL_bool::SDL_TRUE
            {
                bail!("SDL_Vulkan_GetInstanceExtensions");
            }
        }
        let mut v: Vec<*const i8> = vec![ptr::null(); n_names as usize];
        unsafe {
            if SDL_Vulkan_GetInstanceExtensions(
                self.raw(),
                &mut n_names,
                v.as_mut_ptr() as *mut *const _,
            ) != SDL_bool::SDL_TRUE
            {
                bail!("SDL_Vulkan_GetInstanceExtensions");
            }
        }
        Ok(v)
    }
    fn create_vk_surface(
        &self,
        instance: &ash::Instance<V1_0>,
    ) -> Result<vkt::SurfaceKHR, failure::Error> {
        let handle = instance.handle();
        let mut surface: vkt::SurfaceKHR = vkt::SurfaceKHR::null();

        let res = unsafe {
            let win_handle = self.raw();
            SDL_Vulkan_CreateSurface(win_handle, handle, &mut surface)
        };
        if SDL_bool::SDL_TRUE != res {
            Err(format_err!("unimplemented"))
        } else {
            Ok(surface)
        }
    }
}

pub fn sdl_supports_vulkan() -> bool {
    let sdl_version = version();
    let version_num =
        sdl_version.major * 100 + sdl_version.minor * 10 + sdl_version.patch;
    version_num >= 208
}
