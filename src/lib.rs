
#[macro_use]
extern crate failure;

#[allow(unused_imports)]
#[macro_use]
extern crate memoffset;
#[macro_use]
extern crate serde_derive;

pub mod config;
pub mod game;
pub mod platform_system;
pub mod renderer;
pub mod sdl_platform;
use std::error;

// vulkan feature
pub use crate::game::main_loop::MainLoopState;

pub fn get_error_desc<E: error::Error>(e: E) -> String {
    e.description().to_string()
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
