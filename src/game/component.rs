use cgmath::*;
use math::*;
use renderer::material::*;
use std::collections::HashMap;

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

#[derive(Debug, Clone)]
pub struct TransformComponent {
    pub parent: Option<EntityId>,
    pub transform: Decomposed<Vec3, Quaternion<f32>>,
}

impl Default for TransformComponent {
    fn default() -> Self {
        TransformComponent {
            parent: None,
            transform: Decomposed {
                scale: 1.0,
                rot: Quaternion::zero(),
                disp: Vec3::zero(),
            },
        }
    }
}

#[derive(Debug)]
pub struct ComponentManager {
    pub masks: Vec<ComponentMask>,
    pub transforms: HashMap<EntityId, TransformComponent>,
    pub static_meshes: HashMap<EntityId, ()>,
    pub materials: HashMap<EntityId, ResourceId>,
}

impl ComponentManager {
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

    pub fn enumerate_entities<'a>(&'a self) -> EntityIter<'a> {
        EntityIter {
            manager: self,
            i: 0,
        }
    }
}

pub struct EntityIter<'a> {
    manager: &'a ComponentManager,
    i: usize,
}

impl<'a> Iterator for EntityIter<'a> {
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
