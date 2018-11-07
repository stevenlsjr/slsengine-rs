/**
 * bindings for SDL2's vulkan interface
 */

use failure::{Error, Fail};
use sdl2::sys::{SDL_Window, SDL_bool};
use sdl2::video::Window;
use std::os::raw::c_char;
use std::ptr;
use std::ffi::CStr;

extern "C" {
    pub fn SDL_Vulkan_GetInstanceExtensions(
        window: *mut SDL_Window,
        pCount: *mut u32,
        pNames: *mut *const c_char,
    ) -> SDL_bool;
}

///
/// Trait declarations for safe import of sdl_vulkan
pub mod prelude {
    pub use super::GetInstanceExtensions;
}

pub trait GetInstanceExtensions {
    fn vk_instance_extensions(&self) -> Result<Vec<*const i8>, Error>;
    fn vk_instance_extensions_ctsrs(&self) -> Result<Vec<&'static CStr>, Error>{
        let pointers: Vec<*const i8> = self.vk_instance_extensions()?;
        let mut cstrs: Vec<&'static CStr> = Vec::with_capacity(pointers.len());
        for p in pointers {
            let cstring = unsafe {CStr::from_ptr(p)};
            cstrs.push(cstring);
        }
        Ok(cstrs)
    }
}

impl GetInstanceExtensions for Window {
    fn vk_instance_extensions(&self) -> Result<Vec<*const i8>, Error> {
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
                v.as_mut_ptr(),
            ) != SDL_bool::SDL_TRUE
            {
                bail!("SDL_Vulkan_GetInstanceExtensions");
            }

        }
        Ok(v)
    }
}
