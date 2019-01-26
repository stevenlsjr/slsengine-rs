pub use super::built_in_components::*;
use super::component_stores::{Storage, AnyComponentStore, TryGetComponent, GetComponent};
use crate::renderer::traits::*;
use anymap::{Map, any::Any};
use bitflags::bitflags;
use slsengine_entityalloc::*;
use std::{fmt::Debug, ops::Deref, sync::{Arc, RwLock}};


pub trait Component: Debug {
    /// The component mask bitflag identifying the given component
    const MASK: ComponentMask;
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
    pub masks: IndexArray<ComponentMask>,
    pub transforms: Storage<TransformComponent>,
    pub meshes: Storage<MeshComponent>,
    pub materials: Storage<MaterialComponent>,
    custom_stores: AnyComponentStore,
}

impl ComponentManager {
    pub fn new() -> Self {
        let capacity = 255;
        ComponentManager {
            entity_alloc: GenerationalIndexAllocator::with_capacity(capacity),
            masks: IndexArray::with_capacity(capacity),
            transforms: Storage::with_capacity(capacity),
            meshes: Storage::with_capacity(capacity),
            materials: Storage::with_capacity(capacity),
            custom_stores: AnyComponentStore::new()
        }
    }

    /// generates bitmask for entity by components
    pub fn calc_mask(&mut self, entity: Entity) {
        let mut mask = ComponentMask::NONE;
        if self.transforms.read().unwrap().get(*entity).is_some() {
            mask |= ComponentMask::TRANSFORM;
        }
        if self.meshes.read().unwrap().get(*entity).is_some() {
            mask |= ComponentMask::MESH;
        }
        if self.materials.read().unwrap().get(*entity).is_some() {
            mask |= ComponentMask::MATERIAL;
        }
        self.masks.insert(*entity, mask);
    }

    pub fn alloc_entity(&mut self) -> Entity {
        let idx = self.entity_alloc.allocate();
        self.masks.insert(idx, ComponentMask::NONE);
        Entity(idx)
    }
    pub fn dealloc_entity(&mut self, entity: Entity) {
        self.entity_alloc.deallocate(entity.0);
        self.masks.remove(entity.0);
    }

    pub fn entities<'a>(&'a self) -> impl Iterator<Item = Entity> + 'a {
        self.entity_alloc.iter_live().map(|i| Entity(i))
    }
}

impl TryGetComponent for ComponentManager {
    fn try_get_component<C:Component+'static>(&self) -> Option<Storage<C>> {
        use std::any::{TypeId, Any};
        let tid = TypeId::of::<C>();
        if tid == TypeId::of::<TransformComponent>() {
            Some(Any::downcast_ref::<Storage<C>>(&self.transforms).unwrap().clone())
        } else {None}
    }
}