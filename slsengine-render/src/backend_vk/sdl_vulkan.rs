use crate::sdl_platform::*;
/**
 * bindings for SDL2's vulkan interface
 */
use sdl2::{version::version, video::*, VideoSubsystem};

pub fn sdl_supports_vulkan() -> bool {
    let sdl_version = version();
    let version_num =
        sdl_version.major * 100 + sdl_version.minor * 10 + sdl_version.patch;
    version_num >= 208
}

/// Hooks for a vk renderer platform
pub struct VulkanPlatformHooks;

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
