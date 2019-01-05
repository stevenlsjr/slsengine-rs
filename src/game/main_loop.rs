use crate::{game, renderer};
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::video::Window;
use std::{cell::RefCell, time::Instant};

/// State object for main loop information, such as
/// Event handlers and frame timers.
#[derive(Debug)]
pub struct MainLoopState {
    pub is_running: bool,
    pub last_time: Instant,
}

impl MainLoopState {
    pub fn new() -> Self {
        Self::default()
    }
    #[cfg(target_os = "ios")]
    pub fn handle_events<R: renderer::Renderer>(
        &mut self,
        window: &Window,
        event_pump: &RefCell<sdl2::EventPump>,
        renderer: &R,
        world: &mut game::EntityWorld<R>,
    ) {
    }

    #[cfg(not(target_os = "ios"))]
    pub fn handle_events<R: renderer::Renderer>(
        &mut self,
        window: &Window,
        event_pump: &RefCell<sdl2::EventPump>,
        renderer: &R,
        world: &mut game::EntityWorld<R>,
    ) {
        use cgmath::*;
        if world.input_state.is_none() {
            let ep = event_pump.borrow();
            let mouse_state = ep.mouse_state();
            let mousepos =
                Point2::new(mouse_state.x() as f32, mouse_state.y() as f32);
            world.input_state = Some(game::InputState {
                mousepos,
                last_mousepos: mousepos,
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
                Event::Window { win_event, .. } => {
                    if let WindowEvent::Resized(_width, _height) = win_event {
                        let size = window.drawable_size();
                        renderer.on_resize(size);
                    }
                }
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
                        use sdl2::keyboard::{Keycode, Mod};
                        if code == Keycode::R
                            && !repeat
                            && keymod.contains(Mod::LALTMOD)
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

impl Default for MainLoopState {
    fn default() -> Self {
        MainLoopState {
            is_running: false,
            last_time: Instant::now(),
        }
    }
}
