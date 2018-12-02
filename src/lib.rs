#![feature(duration_float)]

extern crate cgmath;
extern crate core;
extern crate rand;
#[macro_use]
extern crate failure;

#[macro_use]
extern crate bitflags;
extern crate gl;

extern crate image;
#[allow(unused_imports)]
#[macro_use]
extern crate memoffset;
extern crate sdl2;

#[cfg(feature = "with-vulkan")]
extern crate vulkano;

extern crate gltf;

#[macro_use]
extern crate serde_derive;
extern crate serde;

pub mod config;
pub mod game;
pub mod renderer;
pub mod renderer_common;
pub mod sdl_platform;

// vulkan feature

use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::video::Window;
use std::{cell::RefCell, error::Error, time::Instant};

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

    pub fn handle_events<R: renderer::Renderer>(
        &mut self,
        window: &Window,
        event_pump: &RefCell<sdl2::EventPump>,
        renderer: &R,
        world: &mut game::EntityWorld,
    ) {
        use cgmath::*;
        if let None = world.input_state {
            let ep = event_pump.borrow();
            let mouse_state = ep.mouse_state();
            let mousepos =
                Point2::new(mouse_state.x() as f32, mouse_state.y() as f32);
            world.input_state = Some(game::InputState {
                mousepos,
                last_mousepos: mousepos.clone(),
            });
        }
        for event in event_pump.borrow_mut().poll_iter() {
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
                        let size = window.drawable_size();
                        renderer.on_resize(size);
                    }
                    _ => {}
                },
                Event::MouseMotion { x, y, .. } => {
                    if let Some(mut input_state) = world.input_state.clone() {
                        input_state.last_mousepos = input_state.mousepos;
                        input_state.mousepos = Point2::new(x as f32, y as f32);
                        world.input_state = Some(input_state);
                    }
                }
                Event::KeyDown {
                    keycode,
                    repeat,
                    keymod,
                    ..
                } => {
                    if let Some(code) = keycode {
                        use sdl2::keyboard::{Keycode, LALTMOD};
                        if code == Keycode::R
                            && !repeat
                            && keymod.contains(LALTMOD)
                        {
                            renderer.flag_shader_recompile();
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

/// application error handling
#[derive(Fail, Debug)]
pub enum AppError {
    #[fail(display = "App error: '{}'", _0)]
    Other(failure::Error),
}

use std::fmt::{Debug, Display};
use std::marker::{Send, Sync};

impl AppError {
    pub fn from_message<D: Display + Debug + Send + Sync + Sized + 'static>(
        message: D,
    ) -> AppError {
        AppError::Other(failure::err_msg(message))
    }
}

pub mod math {
    use cgmath;
    pub type Vec2 = cgmath::Vector2<f32>;
    pub type Vec3 = cgmath::Vector3<f32>;
    pub type Vec4 = cgmath::Vector4<f32>;

    pub type Mat3 = cgmath::Matrix3<f32>;
    pub type Mat4 = cgmath::Matrix4<f32>;
}
