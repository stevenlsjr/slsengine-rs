/**
 * bindings for SDL2's vulkan interface
 */
use sdl2::version::version;

pub fn sdl_supports_vulkan() -> bool {
    let sdl_version = version();
    let version_num =
        sdl_version.major * 100 + sdl_version.minor * 10 + sdl_version.patch;
    version_num >= 208
}
