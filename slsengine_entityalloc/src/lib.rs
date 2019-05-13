extern crate wasm_bindgen;
#[macro_use]
extern crate serde_derive;

pub mod allocator;
pub mod components;

pub mod ffi;
pub mod index_array;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

/// A generational index as described in
/// https://kyren.github.io/2018/09/14/rustconf-talk.html
///
pub use crate::{allocator::*, index_array::*};
