#[macro_use]
extern crate ash;
extern crate glutin;
extern crate slsengine;

use ash::version::V1_0;
use ash::Entry;
use std::rc::Rc;

use slsengine::sdl_platform::*;

struct VulkanPlatformHooks;
impl PlatformBuilderHooks for VulkanPlatformHooks {
    fn build_window(
        &self,
        platform_builder: &PlatformBuilder,
        video_subsystem: &VideoSubsystem,
    ) -> PlatformResult<Window> {
        let title = platform_builder.window_title;
        let window = video_subsystem.window()
    }
}

fn main() {
    let platform = platform().
        build(&VulkanPlatformHooks).unwrap();
    println!("{}", platform);
}
