
use mopa::{self, mopafy};
use std::any::TypeId;
use super::index_array::IndexArray;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
/// Represents an idex array with a dynamic type.
pub trait AnyIndexArray: mopa::Any {
    fn item_typeid(&self) -> TypeId;
}
mopafy!(AnyIndexArray);

impl<T> AnyIndexArray for IndexArray<T> where T: 'static {
    fn item_typeid(&self) -> TypeId {
        TypeId::of::<T>()
    }
}

impl<T> AnyIndexArray for RwLock<IndexArray<T>> where T: 'static {
    fn item_typeid(&self) -> TypeId {
        TypeId::of::<T>()
    }
}



pub struct AnyIndexArraySet {
    map: HashMap<TypeId, Arc<dyn AnyIndexArray>>
}

impl AnyIndexArraySet {
    pub fn new() -> Self {
        AnyIndexArraySet {
            map: HashMap::new()
        }
    }

    pub fn insert<T: 'static>(&mut self, array: IndexArray<T>) -> Option<Arc<AnyIndexArray>> {
        let tid = array.item_typeid();
        let lock = RwLock::new(array);
        self.map.insert(tid, Arc::new(lock) as Arc<dyn AnyIndexArray>)
    }

    pub fn get_by_id(&mut self, typeid: &TypeId) -> Option<Arc<AnyIndexArray>>{
        self.map.get(typeid).cloned()
    }

   
}


#[test]
fn test_any_index_array(){
    use crate::index_array::*;
    use crate::allocator::*;
    let array: IndexArray<i32> = IndexArray::new();
    let array: Box<dyn AnyIndexArray> = Box::new(array);
    assert_eq!(array.item_typeid(), TypeId::of::<i32>());
}

#[test]
fn test_array_set(){
    let mut set = AnyIndexArraySet::new();
    let i32_array = IndexArray::<i32>::new();
    let u32_array = IndexArray::<u32>::new();
    set.insert(i32_array);
    assert!(set.get_by_id(&TypeId::of::<i32>()).is_some());
    assert!(set.get_by_id(&TypeId::of::<u32>()).is_some());
}