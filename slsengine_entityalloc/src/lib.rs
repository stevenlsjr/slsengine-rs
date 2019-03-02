pub mod allocator;
/// Container for a set of index arrays by array item type
///
pub mod any_array;
pub mod index_array;
/// A generational index as described in
/// https://kyren.github.io/2018/09/14/rustconf-talk.html
///
pub use crate::{allocator::*, index_array::*};
