pub use super::built_in_components::*;
use super::component_stores::{
    ComponentIdGen, GetComponent, Storage, TryGetComponent,
};
use crate::renderer::traits::*;
use bitflags::bitflags;
use hibitset::{BitSet, BitSetLike};
use slsengine_entityalloc::*;
use std::{
    any::Any,
    ops::Deref,
    sync::{Arc, RwLock},
};

pub enum StoreType {
    IndexArray,
}

pub trait Component: Any {
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
pub struct ComponentManager<S>
where  S: TryGetComponent {
    pub entity_alloc: GenerationalIndexAllocator,
    pub masks: IndexArray<BitSet>,
    custom_store: S,
    id_table: ComponentIdGen,
}

impl<S> ComponentManager<S> where S: TryGetComponent {
    pub fn new(custom_store: S) -> Self {
        let capacity = 255;
        let mut id_table = ComponentIdGen::new();
        

        ComponentManager {
            entity_alloc: GenerationalIndexAllocator::with_capacity(capacity),
            masks: IndexArray::with_capacity(capacity),
            custom_store,
            id_table,
        }
    }

    /// generates bitmask for entity by components
    pub fn calc_mask(&mut self, entity: Entity) {
        unimplemented!()
    }

    pub fn recalculate_masks<I: Iterator<Item = Entity>>(&mut self, itor: I) {
        for entity in itor {
            self.calc_mask(entity);
        }
    }

    #[inline]
    pub fn id_table(&self) -> &ComponentIdGen {
        &self.id_table
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

    /// returns runtime registered store
    #[inline]
    pub fn get_components<C: Component + 'static>(
        &self,
    ) -> Option<Arc<Storage<C>>> {
        TryGetComponent::try_get_component::<C>(&self.custom_store)
    }

    pub fn entity_mask(&self, entity: Entity) -> &BitSet {
        use lazy_static::lazy_static;
        lazy_static! {
            static ref EMPTY_MASK: BitSet = BitSet::new();
        }

        &self.masks.get(*entity).unwrap_or(&EMPTY_MASK)
    }

    pub fn component_mask<C: Component + 'static>(&self) -> u32 {
        self.id_table.get::<C>().unwrap_or_else(|| {
            panic!("Mask value for component missing. was it registered?")
        })
    }
}
