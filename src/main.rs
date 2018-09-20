extern crate sdl2;


use sdl2::{Sdl, VideoSubsystem};
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::video::Window;
use sdl_platform::{Platform, PlatformBuilder};
use std::error::Error;
use std::rc::Rc;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

pub mod sdl_platform;
pub mod gl_render;


#[cfg(test)]
mod test;


///
/// State object for main loop information, such as
/// Event handlers and frame timers.
#[derive(Debug)]
struct MainLoopState {
    is_running: bool,
}

impl MainLoopState {
    fn new() -> MainLoopState {
        MainLoopState {
            is_running: false,
        }
    }

    fn on_resize(&mut self, _window: &Window, width: i32, height: i32) {
        eprintln!("Hello world!!! {}, {}", width, height)
    }
    fn handle_events(
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


fn game_main() -> Result<(), String> {
    use sdl_platform::{platform, get_error_desc};
    let Platform {window, video_subsystem, event_pump, ..} =
        platform()
            .with_window_size(640, 480)
            .with_window_title("Rust opengl demo")
            .build()?;
    let mut loop_state = MainLoopState::new();
    loop_state.is_running = true;

    while loop_state.is_running {
        loop_state.handle_events(&window, event_pump.borrow_mut().poll_iter());

    }
    Ok(())
}


fn main() {
    if let Err(e) = game_main() {
        eprintln!("game main failed: {}", e);
    }
}
