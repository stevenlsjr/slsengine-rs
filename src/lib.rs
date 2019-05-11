#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
#[allow(unused_imports)]
#[macro_use]
extern crate memoffset;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate smart_default;
#[macro_use]
extern crate specs_derive;



use std::error;
/// application error handling
use std::fmt::{Debug, Display};
use std::marker::{Send, Sync};

pub use crate::{application::AppError, game::main_loop::MainLoopState};
pub use slsengine_render as renderer;


pub mod application;
pub mod assets;
pub mod config;
pub mod game;
pub mod platform_system;
pub mod sdl_platform;

pub fn get_error_desc<E: error::Error>(e: E) -> String {
    e.description().to_string()
}

pub mod math {
    use cgmath;

    pub type Vec2 = cgmath::Vector2<f32>;
    pub type Vec3 = cgmath::Vector3<f32>;
    pub type Vec4 = cgmath::Vector4<f32>;

    pub type Mat3 = cgmath::Matrix3<f32>;
    pub type Mat4 = cgmath::Matrix4<f32>;
}
