extern crate sdl2;



#[allow(unused_imports)]
#[macro_use]
extern crate memoffset;

use std::time::SystemTime;
use std::error::Error;

pub mod renderer;
pub mod renderer_common;

pub mod sdl_platform;

// vulkan feature

#[cfg(feature="with-vulkan")]
#[allow(unused_imports)]
#[macro_use]
extern crate ash;

#[cfg(feature="with-vulkan")]
pub mod renderer_vk;

#[macro_use]
extern crate failure;

use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::video::Window;

pub fn get_error_desc<E: Error>(e: E) -> String {
    e.description().to_string()
}

///
/// State object for main loop information, such as
/// Event handlers and frame timers.
#[derive(Debug)]
pub struct MainLoopState {
    pub is_running: bool,
    pub last_time: SystemTime
}

impl MainLoopState {
    pub fn new() -> MainLoopState {
        MainLoopState { is_running: false,
        last_time: SystemTime::now() }
    }

    pub fn on_resize(&mut self, _window: &Window, width: i32, height: i32) {
        eprintln!("Hello world!!! {}, {}", width, height)
    }
    pub fn handle_events(
        &mut self,
        window: &Window,
        events: sdl2::event::EventPollIterator,
    ) {
        for event in events {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    self.is_running = false;
                }
                Event::Window { win_event, .. } => match win_event {
                    WindowEvent::Resized(width, height) => {
                        self.on_resize(window, width, height);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}

/// application error handling
#[derive(Fail, Debug)]
pub enum AppError {
       #[cfg(feature="with-vulkan")]
    #[fail(display = "vulkan error: {}", _0)]
    VkError(ash::vk::Result),

    #[fail(display = "misc error: '{}'", _0)]
    Misc(String),
}
