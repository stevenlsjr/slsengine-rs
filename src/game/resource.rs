use crate::renderer::Renderer;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Hash)]
pub struct MeshHandle(pub usize);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Hash)]
pub struct TextureHandle(pub usize);

#[derive(Debug)]
pub struct ResourceManager<R: Renderer> {
    pub textures: HashMap<TextureHandle, R::Texture>,
    pub meshes: HashMap<MeshHandle, R::Mesh>,
}

impl<R: Renderer> ResourceManager<R> {
    pub fn new() -> Self {
        ResourceManager::default()
    }
}

impl<R: Renderer> Default for ResourceManager<R> {
    fn default() -> Self {
        ResourceManager {
            textures: HashMap::new(),
            meshes: HashMap::new(),
        }
    }
}
