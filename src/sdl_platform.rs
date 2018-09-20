extern crate sdl2;


use sdl2::{Sdl, VideoSubsystem};
use sdl2::event::{Event, WindowEvent};
use sdl2::video::Window;
use std::cell::RefCell;
use std::error::Error;
use std::fmt;
use std::rc::Rc;


///
/// Container for SDL-level resources, such as subsystems and the
/// primary window.
pub struct Platform {
    pub window: Window,
    pub video_subsystem: VideoSubsystem,
    pub sdl_context: Sdl,
    pub event_pump: Rc<RefCell<sdl2::EventPump>>,
}

impl Platform {

}

impl fmt::Debug for Platform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let addr = {
            let ptr = self.window.raw() as *const _;
            ptr as usize
        };
        write!(f, r#"Platform {{window: 0x{:x}, ..}}"#, addr)
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let title = self.window.title();
        write!(f, r#"Platform for {}"#, title)
    }
}

pub type PlatformResult<T> = Result<T, String>;


/// Constructs a PlatformBuilder struct
pub fn platform() -> PlatformBuilder {
    PlatformBuilder::new()
}

///
/// Builder patter for Platform
pub struct PlatformBuilder {
    window_size: (u32, u32),
    window_title: String,
}

impl PlatformBuilder {
    fn new() -> PlatformBuilder {
        PlatformBuilder {
            window_size: (640, 480),
            window_title: "Window".to_string(),
        }
    }

    pub fn with_window_size(
        &mut self,
        width: u32,
        height: u32,
    ) -> &mut PlatformBuilder {
        self.window_size = (width, height);
        self
    }

    pub fn with_window_title(
        &mut self,
        title: &str,
    ) -> &mut PlatformBuilder {
        self.window_title = title.to_string();
        self
    }

    pub fn build(&self) -> PlatformResult<Platform> {
        use sdl2::video::GLProfile;

        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;
        {
            let gl_attr = video_subsystem.gl_attr();
            gl_attr.set_context_profile(GLProfile::Core);
            gl_attr.set_context_flags().debug().set();
            gl_attr.set_context_version(4, 1);
            gl_attr.set_multisample_buffers(1);
            gl_attr.set_multisample_samples(4);
        }

        let title = self.window_title.as_str();
        let (width, height) = self.window_size;
        let window = video_subsystem
            .window(title, width, height)
            .resizable()
            .opengl()
            .build()
            .map_err(get_error_desc)?;
        {
            let gl_attr = video_subsystem.gl_attr();
            let profile = gl_attr.context_profile();
            let context_version = gl_attr.context_version();
            if profile != GLProfile::Core {
                return Err(format!("profile {:?} is not Core", profile));
            } else if context_version.0 < context_version.0 {
                return Err(format!(
                    "OpenGL version is {}.{}, requested version 3.2",
                    context_version.0,
                    context_version.1
                ));
            }
        }

        let event_pump = sdl_context.event_pump()?;
        Ok(Platform {
            window: window,
            video_subsystem,
            sdl_context,
            event_pump: Rc::new(RefCell::new(event_pump)),
        })
    }
}


pub fn get_error_desc<E: Error>(e: E) -> String {
    String::from(e.description())
}
