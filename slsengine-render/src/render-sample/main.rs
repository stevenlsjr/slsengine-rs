#![cfg_attr(
    not(any(
        feature = "vulkan",
        feature = "dx11",
        feature = "dx12",
        feature = "metal",
        feature = "gl",
        feature = "wgl"
    )),
    allow(dead_code, unused_extern_crates, unused_imports)
)]

use log::debug;
use slsengine_render::backend;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use failure::{format_err, Error};
use hal::{
    buffer, command, format as f,
    format::{AsFormat, ChannelType, Rgba8Srgb as ColorFormat, Swizzle},
    image as i, memory as m, pass,
    pass::Subpass,
    pool,
    prelude::*,
    pso,
    pso::{PipelineStage, ShaderStageFlags, VertexInputRate},
    queue::{QueueGroup, Submission},
    window,
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

fn run<T>(event_loop: EventLoop<T>, window: Window) -> Result<(), Error> {
    event_loop.run(move |event, _, control_flow| match event {
        Event::EventsCleared => {
            // Application update code.

            // Queue a RedrawRequested event.
            window.request_redraw();
        }
        Event::WindowEvent {
            event: WindowEvent::RedrawRequested,
            ..
        } => {
            // Redraw the application.
            //
            // It's preferrable to render in this event rather than in EventsCleared, since
            // rendering in here allows the program to gracefully handle redraws requested
            // by the OS.
        }
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            println!("The close button was pressed; stopping");
            *control_flow = ControlFlow::Exit
        }
        _ => *control_flow = ControlFlow::Poll,
    });
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    run(event_loop, window).unwrap();
}
