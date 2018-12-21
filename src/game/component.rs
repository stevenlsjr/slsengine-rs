pub use super::built_in_components::*;
use crate::math::*;
use crate::renderer::{material::*, traits::*};
use cgmath::*;
use std::{collections::HashMap, rc::Rc};
use std::{fmt::Debug, ops::Index};

use bitflags::bitflags;

pub trait Component: Debug {
    /// The component mask bitflag identifying the given component
    const MASK: ComponentMask;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Hash)]
pub struct EntityId(pub usize);

bitflags! {
    pub struct ComponentMask: u32 {
        const LIVE_ENTITY = 0x2;
        const TRANSFORM = 0x4;
        const STATIC_MESH = 0x5;
        const MATERIAL = 0x5;

    }
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Hash)]
pub struct ResourceId(pub usize);

#[derive(Debug)]
pub struct ComponentManager<R>
where
    R: Renderer,
{
    pub masks: Vec<ComponentMask>,
    pub transforms: HashMap<EntityId, TransformComponent>,
    pub static_meshes: HashMap<EntityId, Rc<R::Mesh>>,
    pub materials: HashMap<EntityId, Material<R::Texture>>,
}

impl<R: Renderer> ComponentManager<R> {
    pub fn new() -> Self {
        ComponentManager {
            masks: vec![ComponentMask::LIVE_ENTITY; 256],
            transforms: HashMap::new(),
            static_meshes: HashMap::new(),
            materials: HashMap::new(),
        }
    }

    pub fn alloc_entity(&mut self) -> EntityId {
        for (id, &mask) in self.masks.iter().enumerate() {
            if (mask & ComponentMask::LIVE_ENTITY).is_empty() {
                return EntityId(id);
            }
        }
        let id = EntityId(self.masks.len());
        self.masks.push(ComponentMask::LIVE_ENTITY);
        id
    }

    pub fn enumerate_entities<'a>(&'a self) -> EntityIter<'a, R> {
        EntityIter {
            manager: self,
            i: 0,
        }
    }
}

pub struct EntityIter<'a, R: Renderer> {
    manager: &'a ComponentManager<R>,
    i: usize,
}

impl<'a, R: Renderer> Iterator for EntityIter<'a, R> {
    type Item = (EntityId, ComponentMask);
    fn next(&mut self) -> Option<Self::Item> {
        let i = self.i;
        self.i += 1;
        let masks = &self.manager.masks;
        if i < masks.len() {
            Some((EntityId(i), masks[i]))
        } else {
            None
        }
    }
}
