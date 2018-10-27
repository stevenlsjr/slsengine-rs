extern crate sdl2;
extern crate cgmath;
extern crate core;

#[allow(unused_imports)]
#[macro_use]
extern crate memoffset;


#[macro_use]
extern crate failure;
// vulkan feature

#[cfg(feature = "with-vulkan")]
#[allow(unused_imports)]
#[macro_use]
extern crate ash;

extern crate gl;

pub mod renderer;
pub mod renderer_common;

pub mod game;
pub mod sdl_platform;



#[cfg(feature = "with-vulkan")]
pub mod renderer_vk;


use std::error::Error;
use std::time::Instant;

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
    pub last_time: Instant,
}

impl MainLoopState {
    pub fn new() -> MainLoopState {
        MainLoopState {
            is_running: false,
            last_time: Instant::now(),
        }
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
    #[cfg(feature = "with-vulkan")]
    #[fail(display = "vulkan error: {}", _0)]
    VkError(ash::vk::Result),

    #[fail(display = "misc error: '{}'", _0)]
    Misc(String),
}
