use super::ash;
use ash::extensions::{DebugReport, Surface};

#[cfg(target_os = "macos")]
use ash::extensions::MacOSSurface;
#[cfg(target_os = "windows")]
use ash::extensions::Win32Surface;
#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
use ash::extensions::XlibSurface;

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
