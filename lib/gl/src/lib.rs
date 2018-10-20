#![allow(unused_parens, non_camel_case_types)]

/// include gl bindings file 
/// 
pub mod gl {
include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

pub use self::gl::*;  