use slsengine_entityalloc::{GenerationalIndex, IndexArray};
use std::ops::Deref;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResourceHandle(pub GenerationalIndex);

impl Deref for ResourceHandle {
    type Target = GenerationalIndex;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
