use super::component::{Component, ComponentList};
use mopa::{self, mopafy};
use std::any::TypeId;
use std::collections::HashMap;
use std::fmt;
use std::ops::Deref;
use std::sync::{Arc, RwLock};

    #[derive(Debug)]
pub struct Storage<C: Component> {
    lock: RwLock<ComponentList<C>>,
}

impl<C: Component> Storage<C> {
    pub fn new() -> Self {
        Storage {
            lock: RwLock::new(ComponentList::new()),
        }
    }
}

impl<C: Component> Deref for Storage<C> {
    type Target = RwLock<ComponentList<C>>;
    fn deref(&self) -> &Self::Target {
        &self.lock
    }
}

pub trait AnyStorage: mopa::Any {}
mopafy!(AnyStorage);

impl<C: Component> AnyStorage for Storage<C> {}

/// Provider of a specifically typed component
pub trait GetComponent<C: Component> {
    fn get_component(&self) -> Arc<Storage<C>>;
}

/// Can try to retrieve a component list of an arbitrary type
pub trait TryGetComponent {
    /// Returns Some component list if storage object contains it.
    fn try_get_component<C: Component>(&self) -> Option<Arc<Storage<C>>>;
}

/// A dummy component store that provides no component lists
#[derive(Clone, PartialEq, Debug, Hash)]
pub struct NullComponentStore;

impl TryGetComponent for NullComponentStore {
    fn try_get_component<C: Component>(&self) -> Option<Arc<Storage<C>>> {None}

}


#[cfg(test)]
mod test {
 
    #[test]
    fn test_any_store() {
        unimplemented!()
    }

    

}
/// Generates unique bitset mask values for a componenet
pub struct ComponentIdGen {
    lut: HashMap<TypeId, u32>,
}

impl fmt::Debug for ComponentIdGen {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ComponentIdGen").finish()
    }
}

impl ComponentIdGen {
    pub fn new() -> Self {
        ComponentIdGen {
            lut: HashMap::new(),
        }
    }

    pub fn get<C: Component + 'static>(&self) -> Option<u32> {
        self.get_id(&TypeId::of::<C>())
    }

    pub fn get_id(&self, id: &TypeId) -> Option<u32> {
        self.lut.get(id).cloned()
    }

    pub fn get_or_insert<C: Component + 'static>(&mut self) -> u32 {
        self.get_or_insert_id(TypeId::of::<C>())
    }

    pub fn get_or_insert_id(&mut self, tid: TypeId) -> u32 {
        if let Some(&index) = self.lut.get(&tid) {
            index
        } else {
            let index = self.lut.len() as u32;
            self.lut.insert(tid, index);
            println!("insert id {:?}, {}", tid, index);
            index
        }
    }
}
