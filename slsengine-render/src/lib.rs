#[macro_use]
extern crate specs_derive;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate smart_default;
#[macro_use]
extern crate memoffset;

#[cfg(target_os="ios")]
pub mod ios_run;
#[cfg(target_arch = "ios")]
pub mod ios;


pub mod camera;
pub mod color;
pub mod components;
pub mod material;
pub mod mesh;
pub mod model;
pub mod traits;

pub mod draw;

pub use self::color::{color4f, ColorRGBA};
pub use self::{camera::*, mesh::*, traits::*};
#[cfg(target_arch = "wasm32")]
pub mod web_entry; 


pub(crate) mod math {
    use cgmath;

    pub type Vec2 = cgmath::Vector2<f32>;
    pub type Vec3 = cgmath::Vector3<f32>;
    pub type Vec4 = cgmath::Vector4<f32>;

    pub type Mat3 = cgmath::Matrix3<f32>;
    pub type Mat4 = cgmath::Matrix4<f32>;
}
