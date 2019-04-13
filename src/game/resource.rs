use crate::renderer::Renderer;
use std::collections::HashMap;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Hash)]
pub struct MeshHandle(pub usize);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Hash)]
pub struct TextureHandle(pub usize);

#[derive(Fail, Debug)]
pub enum ResourceError {
    #[fail(display = "failed to fetch resource")]
    FetchError(failure::Error),
}

pub type ResourceResult<T> = Result<T, ResourceError>;

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

pub trait ResourceFetcher<H> {
    type Resource;
    fn fetch(&self, handle: H) -> Option<&Self::Resource>;
}

impl<R: Renderer> ResourceFetcher<MeshHandle> for ResourceManager<R> {
    type Resource = R::Mesh;
    fn fetch(&self, handle: MeshHandle) -> Option<&Self::Resource> {
        self.meshes.get(&handle)
    }
}

impl<R: Renderer> ResourceFetcher<TextureHandle> for ResourceManager<R> {
    type Resource = R::Texture;
    fn fetch(&self, handle: TextureHandle) -> Option<&Self::Resource> {
        self.textures.get(&handle)
    }
}
