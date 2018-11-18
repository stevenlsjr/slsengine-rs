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


pub fn sdl_supports_vulkan() -> bool {
    let sdl_version = version();
    let version_num =
        sdl_version.major * 100 + sdl_version.minor * 10 + sdl_version.patch;
    version_num >= 208
}
