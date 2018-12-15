use super::objects::*;
use failure;
use gl;
use image::{self, DynamicImage};

#[derive(Debug)]
pub struct TextureObjects {
    ids: Vec<u32>,
}
use std::iter::FromIterator;

pub trait FromImage<Image> {
    fn load_from_image(&mut self, image: &Image) -> Result<(), failure::Error>;
}

impl TextureObjects {
    pub fn new(len: usize) -> Result<TextureObjects, ObjectError> {
        let mut ids: Vec<u32> = vec![0; len];

        unsafe {
            gl::GenTextures(len as i32, ids.as_mut_ptr());
        }

        Ok(TextureObjects { ids })
    }

    /// takes ownership of the texture objects, and returns a FromIter of GlTextures
    pub fn into_individual_textures<B>(mut self) -> B
    where
        B: FromIterator<GlTexture>,
    {
        use std::mem::swap;

        let mut ids = Vec::new();
        swap(&mut self.ids, &mut ids);

        ids.iter().map(|&id| GlTexture { id }).collect()
    }

    pub fn ids(&self) -> &[u32] {
        &self.ids
    }

    pub fn len(&self) -> usize {
        self.ids.len()
    }
}

impl Drop for TextureObjects {
    fn drop(&mut self) {
        if self.len() < 1 {
            return;
        }
        unsafe {
            gl::DeleteTextures(self.ids.len() as i32, self.ids.as_mut_ptr());
        }

        self.ids.clear();
    }
}

#[derive(Debug, PartialEq)]
pub struct GlTexture {
    pub id: u32,
}

impl GlTexture {
    pub fn new() -> Result<Self, failure::Error> {
        let mut id: u32 = 0;
        unsafe {
            gl::GenTextures(1, &mut id as *mut _);
        }
        Ok(GlTexture { id })
    }
}

impl FromImage<gltf::image::Data> for GlTexture
{
    fn load_from_image(&mut self, image: &gltf::image::Data) -> Result<(), failure::Error> {
        use gltf::image::*;
        let format = match image.format {
            Format::R8 => gl::RED,
            Format::R8G8 => gl::RG,
            Format::R8G8B8 => gl::RGB,
            Format::R8G8B8A8 => gl::RGBA,
        };
        let ptr = image.pixels.as_ptr();

        let internal_format = format;

        unsafe {
            gl::BindTexture(self.id, gl::TEXTURE_2D);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                internal_format as i32,
                image.width as i32,
                image.height as i32,
                0,
                format,
                gl::UNSIGNED_BYTE,
                ptr as *const _,
            );
            gl::BindTexture(0, gl::TEXTURE_2D);
        }
        Ok(())
    }
}

impl Drop for GlTexture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &mut self.id as *mut _);
        }
        self.id = 0;
    }
}