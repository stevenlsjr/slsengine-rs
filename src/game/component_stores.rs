use super::component::{Component, ComponentList};
use std::sync::{Arc, RwLock};
use std::ops::Deref;
use anymap::{AnyMap};


/// Newtype for a component array store. Wraps an IndexArray in an
/// Arc'ed RwLock
#[derive(Debug)]
pub struct Storage<C: Component>(pub Arc<RwLock<ComponentList<C>>>);
impl<C: Component> Clone for Storage<C> {
    fn clone(&self) -> Self {
        Storage(self.0.clone())
    }
}

impl<C: Component> Storage<C> {
    pub fn new()-> Self {
        Storage::with_array(ComponentList::new())
    }
    pub fn with_capacity(capacity: usize) ->Self {
        Storage::with_array(ComponentList::with_capacity(capacity))
    }

    pub fn with_array(list: ComponentList<C>) -> Self {
        Storage(Arc::new(RwLock::new(list)))
    }
}
impl<C: Component> Deref for Storage<C> {
    type Target = Arc<RwLock<ComponentList<C>>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait GetComponent<C: Component> {
    fn get_component(&self) -> Storage<C>;
}

pub trait TryGetComponent {
    fn try_get_component<C:Component+'static>(&self) -> Option<Storage<C>>;
}

/// A map that contains up to one ComponentStore for each
/// component type.
#[derive(Debug)]
pub struct AnyComponentStore {
    map: AnyMap
}

impl AnyComponentStore {
    /// Constructs a new store
    pub fn new() -> Self {
        AnyComponentStore {map: AnyMap::new()}
    }

    /// Inserts a preexisting list into the map
    pub fn insert_store<C: 'static>(&mut self, storage: Storage<C>) where C: Component {
        self.map.insert(storage);
    }

}

impl Default for AnyComponentStore {
    fn default() -> Self {
        Self::new()
    }
}

impl TryGetComponent for AnyComponentStore {
    fn try_get_component<C:Component+'static>(&self) -> Option<Storage<C>> {
        self.map.get::<Storage<C>>().cloned()
    }
}




#[cfg(test)]
mod test {
    use super::super::built_in_components::*;
    use super::*;
    use std::any::{Any, TypeId};
    pub struct Mock {
        transforms: Storage<TransformComponent>,
    }
    impl GetComponent<TransformComponent> for Mock {
        fn get_component(&self) -> Storage<TransformComponent> {
            self.transforms.clone()
        }
    }

   
    pub fn mock_component_store() -> Mock {
        use slsengine_entityalloc::IndexArray;

        Mock {
            transforms: Storage::new(),
        }
    }
    #[test]
    fn test_any_store(){
        let mut store = AnyComponentStore::new();
        store.insert_store(Storage::<TransformComponent>::new());
        assert!(store.try_get_component::<TransformComponent>().is_some());
        assert!(store.try_get_component::<MeshComponent>().is_none());
    }

    #[test]
    fn test_component_store() {
        use super::super::built_in_components::{MeshComponent, TransformComponent};
        use crate::renderer::Mesh;
        let store = test::mock_component_store();
        assert!((store.get_component() as Storage<TransformComponent>).read().is_ok());
    }

}
