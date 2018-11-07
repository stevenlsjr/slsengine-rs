#[cfg(feature = "with-vulkan")]
#[allow(unused_imports)]
#[macro_use]
pub extern crate ash;
pub extern crate cgmath;
extern crate core;
#[macro_use]
extern crate failure;
pub extern crate gl;
pub extern crate image;
#[allow(unused_imports)]
#[macro_use]
extern crate memoffset;
pub extern crate sdl2;

// vulkan feature

use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::video::Window;
use std::error::Error;
use std::time::Instant;

pub mod renderer;
pub mod renderer_common;

pub mod game;
pub mod sdl_platform;

#[cfg(feature = "with-vulkan")]
pub mod renderer_vk;

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

    pub fn on_resize(
        &mut self,
        window: &Window,
        width: i32,
        height: i32,
        renderer: &renderer::Renderer,
    ) {
        eprintln!("Hello world!!! {}, {}", width, height);
        renderer.on_resize(window, (width as u32, height as u32));
    }
    pub fn handle_events(
        &mut self,
        window: &Window,
        events: sdl2::event::EventPollIterator,
        renderer: &renderer::Renderer,
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
                        self.on_resize(window, width, height, renderer);
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

    #[fail(display = "Ash instance error: '{}'", _0)]
    AshInstanceError(#[fail(cause)] ash::InstanceError),

    #[fail(display = "App error: '{}'", _0)]
    Other(failure::Error)
}

use std::fmt::{Display, Debug};
use std::marker::{Send, Sync};
use std::clone::Clone;


impl AppError {

    pub fn from_message<D: Display + Debug + Send + Sync + Sized + 'static>(message: D) -> AppError {
        AppError::Other(failure::err_msg(message))
    }
}

