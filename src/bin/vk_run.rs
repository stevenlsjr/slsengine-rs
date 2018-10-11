#![allow(unused_imports)]
#![allow(unused_variables)]

extern crate ash;
extern crate slsengine;

use ash::Entry;
use slsengine::renderer_vk::*;
use ash::vk::PhysicalDevice;
use slsengine::sdl_platform::*;
use ash::version::*;
use ash::*;

pub type AppEntry = Entry<V1_0>;
struct VulkanPlatformHooks;

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



fn main() {
    use std::thread;
    use std::time::Duration;
    let platform = platform().build(&VulkanPlatformHooks).unwrap();
    let entry = Entry::new().unwrap();
    let instance = make_instance(&entry).unwrap();
    let phys_dev = pick_physical_device(&instance).expect("Something's not right");

    let mut main_loop = slsengine::MainLoopState::new();
    main_loop.is_running = true;
    while main_loop.is_running {
        main_loop.handle_events(
            &platform.window,
            platform.event_pump.borrow_mut().poll_iter(),
        );
        thread::sleep(Duration::from_millis(16));
    }
}
