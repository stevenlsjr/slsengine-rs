//!
//! Array of struct components container
//!
//!

pub trait Component {}
pub trait ComponentStore<T>
where
    T: Component,
{
}
pub struct AnyComponentMap {}
