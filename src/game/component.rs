pub use super::built_in_components::*;
use crate::renderer::traits::*;
use bitflags::bitflags;
use slsengine_entityalloc::*;
use std::{
    fmt::Debug,
    ops::{Deref, Index},
};

pub trait Component: Debug {
    /// The component mask bitflag identifying the given component
    const MASK: ComponentMask;
}

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
    pub transforms: IndexArray<TransformComponent>,
    pub meshes: IndexArray<MeshComponent>,
    pub materials: IndexArray<MaterialComponent>,
}

impl ComponentManager {
    pub fn new() -> Self {
        let capacity = 255;
        ComponentManager {
            entity_alloc: GenerationalIndexAllocator::with_capacity(capacity),
            masks: IndexArray::with_capacity(capacity),
            transforms: IndexArray::with_capacity(capacity),
            meshes: IndexArray::with_capacity(capacity),
            materials: IndexArray::with_capacity(capacity),
        }
    }

    /// generates bitmask for entity by components
    pub fn calc_mask(&mut self, entity: Entity) {
        let mut mask = ComponentMask::NONE;
        if self.transforms.get(*entity).is_some() {
            mask |= ComponentMask::TRANSFORM;
        }
        if self.meshes.get(*entity).is_some() {
            mask |= ComponentMask::MESH;
        }
        if self.materials.get(*entity).is_some() {
            mask |= ComponentMask::MATERIAL;
        }
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
}

//-- Get Component trait & impl

pub trait GetComponents<C: Component> {
    fn get_components(&self) -> &IndexArray<C>;
    fn get_components_mut(&mut self) -> &mut IndexArray<C>;
}

impl GetComponents<TransformComponent> for ComponentManager {
    fn get_components(&self) -> &IndexArray<TransformComponent> {
        &self.transforms
    }
    fn get_components_mut(&mut self) -> &mut IndexArray<TransformComponent> {
        &mut self.transforms
    }
}
