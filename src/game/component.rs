use cgmath::*;
use math::*;
use std::collections::HashMap;
use std::iter::Filter;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Hash)]
pub struct EntityId(pub usize);

bitflags! {
    pub struct ComponentMask: u32 {
        const LIVE_ENTITY = 0x2;
        const TRANSFORM = 0x4;
        const STATIC_MESH = 0x5;

    }
}


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
}

impl ComponentManager {
    pub fn new() -> Self {
        ComponentManager {
            masks: vec![ComponentMask::LIVE_ENTITY; 256],
            transforms: HashMap::new(),
            static_meshes: HashMap::new(),
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
}
