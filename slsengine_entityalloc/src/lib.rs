extern crate wasm_bindgen;
#[macro_use]
extern crate serde_derive;

pub mod allocator;
/// Container for a set of index arrays by array item type
///
pub mod any_array;
pub mod ffi;
pub mod index_array;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

/// A generational index as described in
/// https://kyren.github.io/2018/09/14/rustconf-talk.html
///
pub use crate::{allocator::*, index_array::*};
