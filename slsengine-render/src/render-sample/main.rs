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
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    env_logger::init();
    let mut state =
        DemoState::new("gfx demo", LogicalSize::new(480.0, 480.0)).unwrap();
    state.run();
}

struct DemoState {
    event_loop: EventLoop<()>,
    window: Window,
}
impl DemoState {
    pub fn new<T: Into<String>>(
        title: T,
        size: LogicalSize,
    ) -> Result<Self, Error> {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(title.into())
            .with_inner_size(size)
            .build(&event_loop)?;

        Ok(DemoState { event_loop, window })
    }
    pub fn run(mut self) -> ! {
        let DemoState {window, event_loop} = self;
        event_loop.run(move |event, _, control_flow| {
            match event {
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
            }
        });
    }
}

static COMP_SHADER_SRC: &str = r#"
#version 450

layout(local_size_x=64, local_size_y=1, local_size_z=1) in;

layout(set=0, binding=0) buffer Data {
    uint data[];
} data;

void main(){
    uint idx = gl_GlobalInvocationID.x;
    data.data[idx] *= 12;
}
"#;
