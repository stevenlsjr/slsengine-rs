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

pub trait GetComponent<C: Component> {
    fn get_component(&self) -> Arc<Storage<C>>;
}

pub trait TryGetComponent<C: Component> {
    fn try_get_component(&self) -> Option<Arc<Storage<C>>>;
}

/// A map that contains up to one ComponentStore for each
/// component type.
pub struct AnyStorageMap {
    map: HashMap<TypeId, Arc<AnyStorage>>,
}

impl fmt::Debug for AnyStorageMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("AnyStorageMap").finish()
    }
}

impl AnyStorageMap {
    /// Constructs a new store
    pub fn new() -> Self {
        AnyStorageMap {
            map: HashMap::new(),
        }
    }

    /// Inserts a preexisting list into the map
    pub fn insert_store<C: 'static>(&mut self, storage: Arc<Storage<C>>)
    where
        C: Component,
    {
        self.map.insert(TypeId::of::<C>(), storage);
    }

    pub fn keys<'a>(&'a self) -> impl Iterator<Item = TypeId> + 'a {
        self.map.keys().cloned()
    }
}

impl<C: Component + 'static> TryGetComponent<C> for AnyStorageMap {
    fn try_get_component(&self) -> Option<Arc<Storage<C>>> {
        self.map
            .get(&TypeId::of::<C>())
            .cloned()
            .and_then((|a| {
                let store: & AnyStorage = a.deref();
                store.downcast_ref()
            }))
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
        let mut store = AnyStorageMap::new();
        store.insert_store(Storage::<TransformComponent>::new());
        assert!(TryGetComponent::<TransformComponent>::try_get_component(
            &store
        )
        .is_some());
        assert!(TryGetComponent::<MeshComponent>::try_get_component(&store)
            .is_none());
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
            println!("insert id {:?}, {}", tid, index);
            index
        }
    }
}
