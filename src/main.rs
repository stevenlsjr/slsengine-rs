extern crate sdl2;


mod sdl_platform;
use sdl_platform::{Platform, PlatformBuilder};

use std::rc::Rc;


#[cfg(test)]
mod test;

use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::video::Window;
use sdl2::{Sdl, VideoSubsystem};
use std::error::Error;

struct WindowsAndContexts {
    window: Window,
    video_subsystem: VideoSubsystem,
    sdl_context: Sdl,
}

#[derive(Debug)]
struct MainLoopState {
    is_running: bool,
}

impl MainLoopState {
    fn new(platform: Rc<Platform>) -> MainLoopState {
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
    let platform =
        Rc::new(PlatformBuilder::new().with_window_size(640, 480)
        .with_window_title("Rust opengl demo")
        .build()?);
    let mut loop_state = MainLoopState::new(platform.clone());

    println!("{}", platform.as_ref());
    loop_state.is_running = true;
    
    while loop_state.is_running {
        let mut event_pump = platform.event_pump.borrow_mut();
        loop_state.handle_events(&platform.window, event_pump.poll_iter());
    }

    Ok(())
}


fn main() {
    if let Err(e) = game_main() {
        eprintln!("game main failed: {}", e);
    }
}
