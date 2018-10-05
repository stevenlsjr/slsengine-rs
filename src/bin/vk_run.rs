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
        let mut wb = make_window_builder(platform_builder, video_subsystem);
        let window = video_subsystem.window()
    }
}

fn main() {
    let platform = platform().
        build(&VulkanPlatformHooks).unwrap();
    println!("{}", platform);
}
