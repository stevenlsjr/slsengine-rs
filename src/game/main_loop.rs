use std::{
    cell::RefCell,
    time::{Duration, Instant},
};

use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::video::Window;

use crate::game::InputState;
use crate::{game, renderer};

/// State object for main loop information, such as
/// Event handlers and frame timers.
#[derive(Debug)]
pub struct MainLoopState {
    pub is_running: bool,
    pub last_time: Instant,
}

pub struct FrameTick {
    pub delta: Duration,
    pub last_time: Instant,
}

impl MainLoopState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }
    pub fn set_is_running(&mut self, is_running: bool) {
        self.is_running = is_running;
    }

    pub fn start(&mut self) {
        self.last_time = Instant::now();
        self.is_running = true;
        eprintln!("starting loop {:?}", self);
    }

    /// updates time on game loop clock. Returns a FrameTick struct, which provides
    /// the delta time as a duration, as well as the last_time value tick_frame reset
    pub fn tick_frame(&mut self) -> FrameTick {
        let last_time = self.last_time;
        let now = Instant::now();
        let delta = now - last_time;
        self.last_time = now;
        FrameTick { delta, last_time }
    }

    fn create_input_state(
        &mut self,
        event_pump: &RefCell<sdl2::EventPump>,
    ) -> InputState {
        use cgmath::*;

        let ep = event_pump.borrow();
        let mouse_state = ep.mouse_state();
        let mousepos =
            Point2::new(mouse_state.x() as f32, mouse_state.y() as f32);
        game::InputState {
            mousepos,
            last_mousepos: mousepos,
        }
    }

    pub fn handle_events<R: renderer::Renderer>(
        &mut self,
        window: &Window,
        event_pump: &RefCell<sdl2::EventPump>,
        renderer: &R,
        world: &mut game::WorldManager,
    ) {
        use cgmath::*;

        let mut input_state: InputState = {
            let opt: Option<InputState> = world.read_input_state().clone();
            opt.unwrap_or_else(|| self.create_input_state(event_pump))
        };
        for event in event_pump.borrow_mut().poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    self.is_running = false;
                }

                Event::Window {
                    win_event: WindowEvent::Resized(_width, _height),
                    ..
                } => {
                    let size = window.drawable_size();
                    renderer.on_resize(size);
                }
                Event::MouseMotion { x, y, .. } => {
                    input_state.last_mousepos = input_state.mousepos;
                    input_state.mousepos = Point2::new(x as f32, y as f32);
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
        {
            *world.write_input_state() = Some(input_state);
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
