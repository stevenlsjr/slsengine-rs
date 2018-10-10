extern crate sdl2;
#[macro_use]
extern crate ash;

#[macro_use]
extern crate memoffset;
use std::error::Error;

pub mod renderer;

pub mod sdl_platform;

//#[cfg(feature="with-vulkan")]
pub mod renderer_vk;

#[macro_use]
extern crate failure;
//#[macro_use] extern crate serde_derive;

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
}

impl MainLoopState {
    pub fn new() -> MainLoopState {
        MainLoopState { is_running: false }
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
