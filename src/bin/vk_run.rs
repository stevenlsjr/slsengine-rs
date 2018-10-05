extern crate ash;
extern crate glutin;

use ash::Entry;
use ash::version::V1_0;
use ash::vk;
use glutin::dpi::*;
use std::default::Default;

fn main() {
    let mut event_loop = glutin::EventsLoop::new();

    let wb = glutin::WindowBuilder::new()
        .with_title("Hello vulkan")
        .with_dimensions(LogicalSize::new(640.0, 480.0));

    let window = wb.build(&event_loop);


    let mut is_running = true;
    while is_running {
        use glutin::{Event, WindowEvent};
        event_loop.poll_events(|event| {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {is_running = false;}
                    _ => {}
                },
                _ => {}
            };
        });
    }
}
