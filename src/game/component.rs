pub use super::built_in_components::*;
use super::component_stores::{
    AnyComponentStore, ComponentIdGen, GetComponent, Storage, TryGetComponent,
};
use crate::renderer::traits::*;
use anymap::{any::Any, Map};
use bitflags::bitflags;
use hibitset::{BitSet, BitSetLike};
use slsengine_entityalloc::*;
use std::{
    fmt::Debug,
    ops::Deref,
    sync::{Arc, RwLock},
};

pub enum StoreType {
    IndexArray,
}

pub trait Component: Debug {
    /// The component mask bitflag identifying the given component
    const STORE: StoreType = StoreType::IndexArray;
}

pub type ComponentList<C> = IndexArray<C>;

bitflags! {
    pub struct ComponentMask: u32 {
        const NONE = 0x0;
        const LIVE_ENTITY = 0x2;
        const TRANSFORM = 0x4;
        const MESH = 0x5;
        const MATERIAL = 0x5;

    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Hash)]
pub struct Entity(pub GenerationalIndex);

impl Entity {
    #[inline]
    pub fn index(&self) -> usize {
        self.0.index()
    }
}

impl Deref for Entity {
    type Target = GenerationalIndex;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Hash)]
pub struct ResourceId(pub usize);

#[derive(Debug)]
pub struct ComponentManager {
    pub entity_alloc: GenerationalIndexAllocator,
    pub masks: IndexArray<BitSet>,
    pub transforms: Storage<TransformComponent>,
    pub meshes: Storage<MeshComponent>,
    pub materials: Storage<MaterialComponent>,
    custom_stores: AnyComponentStore,
    id_table: ComponentIdGen,
}

impl ComponentManager {
    pub fn new() -> Self {
        let capacity = 255;
        let mut id_table = ComponentIdGen::new();
        id_table.get_or_insert::<TransformComponent>();
        id_table.get_or_insert::<MeshComponent>();
        id_table.get_or_insert::<MaterialComponent>();
        ComponentManager {
            entity_alloc: GenerationalIndexAllocator::with_capacity(capacity),
            masks: IndexArray::with_capacity(capacity),
            transforms: Storage::with_capacity(capacity),
            meshes: Storage::with_capacity(capacity),
            materials: Storage::with_capacity(capacity),
            custom_stores: AnyComponentStore::new(),
            id_table,
        }
    }

    /// generates bitmask for entity by components
    pub fn calc_mask(&mut self, entity: Entity) {
        use anymap::raw::RawMap;
        use std::any::Any;
        let mut mask = BitSet::new();

        if self.transforms.read().unwrap().get(*entity).is_some() {
            mask.add(
                self.id_table
                    .get::<TransformComponent>()
                    .expect("BUG!, no mask id for Component"),
            );
        }
        if self.meshes.read().unwrap().get(*entity).is_some() {
            mask.add(
                self.id_table
                    .get::<MeshComponent>()
                    .expect("BUG!, no mask id for Component"),
            );
        }
        if self.materials.read().unwrap().get(*entity).is_some() {
            mask.add(
                self.id_table
                    .get::<MaterialComponent>()
                    .expect("BUG!, no mask id for Component"),
            );
        }

        for i in self.custom_stores.keys() {
            let id = self.id_table.get_or_insert_id(i);
            mask.add(id);
        }
        self.masks.insert(*entity, mask);
    }

    pub fn register<C: Component + 'static>(&mut self) {
        self.custom_stores.insert_store::<C>(Storage::new());
        self.id_table.get_or_insert::<C>();
    }

    pub fn alloc_entity(&mut self) -> Entity {
        let idx = self.entity_alloc.allocate();
        Entity(idx)
    }
    pub fn dealloc_entity(&mut self, entity: Entity) {
        self.entity_alloc.deallocate(entity.0);
        self.masks.remove(entity.0);
    }

    pub fn entities<'a>(&'a self) -> impl Iterator<Item = Entity> + 'a {
        self.entity_alloc.iter_live().map(|i| Entity(i))
    }

    //-- Component retreival

    /// Returns transforms store
    #[inline]
    pub fn transforms(&self) -> Storage<TransformComponent> {
        self.transforms.clone()
    }

    /// Returns static mesh store
    #[inline]
    pub fn meshes(&self) -> Storage<MeshComponent> {
        self.meshes.clone()
    }

    /// Returns static mesh store
    #[inline]
    pub fn materials(&self) -> Storage<MaterialComponent> {
        self.materials.clone()
    }

    /// returns runtime registered store
    #[inline]
    pub fn other_components<C: Component + 'static>(
        &self,
    ) -> Option<Storage<C>> {
        TryGetComponent::<C>::try_get_component(&self.custom_stores)
    }

    pub fn mask_for<C: Component + 'static>(&self) -> u32 {
        self.id_table.get::<C>().unwrap_or_else(|| {
            panic!("Mask value for component missing. was it registered?")
        })
    }
}
