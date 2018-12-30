pub mod allocator;
pub mod index_array;
/// A generational index as described in
/// https://kyren.github.io/2018/09/14/rustconf-talk.html
///
pub use crate::{allocator::*, index_array::*};
