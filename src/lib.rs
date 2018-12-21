#![feature(duration_float)]
#![feature(const_fn)]

#[macro_use]
extern crate bitflags;

extern crate cgmath;
extern crate core;
extern crate rand;

#[macro_use]
extern crate log;
#[macro_use]
extern crate failure;
extern crate genmesh;
extern crate gl;
extern crate gltf;
extern crate image;
#[allow(unused_imports)]
#[macro_use]
extern crate memoffset;
extern crate sdl2;
#[macro_use]
extern crate serde_derive;
extern crate serde;
#[cfg(feature = "with-vulkan")]
extern crate vulkano;

pub mod config;
pub mod game;
pub mod renderer;
pub mod sdl_platform;
pub mod system;
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
