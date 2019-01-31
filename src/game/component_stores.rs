use super::component::{Component, ComponentList};
use anymap::AnyMap;
use std::any::TypeId;
use std::collections::HashMap;
use std::fmt;
use std::ops::Deref;
use std::sync::{Arc, RwLock};

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
    pub fn new() -> Self {
        Storage::with_array(ComponentList::new())
    }
    pub fn with_capacity(capacity: usize) -> Self {
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

pub trait TryGetComponent<C: Component> {
    fn try_get_component(&self) -> Option<Storage<C>>;
}

/// A map that contains up to one ComponentStore for each
/// component type.
#[derive(Debug)]
pub struct AnyComponentStore {
    map: AnyMap,
}

impl AnyComponentStore {
    /// Constructs a new store
    pub fn new() -> Self {
        AnyComponentStore { map: AnyMap::new() }
    }

    /// Inserts a preexisting list into the map
    pub fn insert_store<C: 'static>(&mut self, storage: Storage<C>)
    where
        C: Component,
    {
        self.map.insert(storage);
    }

    pub fn keys<'a>(&'a self) -> impl Iterator<Item = TypeId> + 'a {
        self.map.as_ref().iter().map(|a| a.get_type_id())
    }
}

impl<C: Component + 'static> TryGetComponent<C> for AnyComponentStore {
    fn try_get_component(&self) -> Option<Storage<C>> {
        self.map.get().cloned()
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
    fn test_any_store() {
        let mut store = AnyComponentStore::new();
        store.insert_store(Storage::<TransformComponent>::new());
        assert!(TryGetComponent::<TransformComponent>::try_get_component(&store).is_some());
        assert!(TryGetComponent::<MeshComponent>::try_get_component(&store).is_none());
    }

    #[test]
    fn test_component_store() {
        use super::super::built_in_components::{
            MeshComponent, TransformComponent,
        };
        use crate::renderer::Mesh;
        let store = test::mock_component_store();
        assert!((store.get_component() as Storage<TransformComponent>)
            .read()
            .is_ok());
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
            index
        }
    }
}
