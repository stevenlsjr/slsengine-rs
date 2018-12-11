use super::gl_renderer::*;
use super::objects::*;
use failure;
use gl;
use image;
use std::path::Path;

#[derive(Debug)]
pub struct TextureObjects {
    ids: Vec<u32>,
}
use std::iter::FromIterator;

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

    pub fn load_image(
        &mut self,
        image: image::DynamicImage,
    ) -> Result<(), failure::Error> {
        use image::*;
        let (input_format, _depth) = match image.color() {
            ColorType::Gray(depth) => (gl::RED, depth),
            ColorType::GrayA(depth) => (gl::RG, depth),
            ColorType::RGB(depth) => (gl::RGB, depth),
            ColorType::RGBA(depth) => (gl::RGBA, depth),
            ColorType::BGR(depth) => (gl::RGB, depth),
            ColorType::BGRA(depth) => (gl::RGBA, depth),

            other => bail!("invalid color format {:?}", other),
        };

        let output_format = match input_format {
            gl::RG => gl::RG,
            gl::RGB => gl::RGB,
            gl::RGBA => gl::RG,
            gl::RED => gl::RED,
            other => gl::RGB
        };
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                output_format as i32,
                image.width() as i32,
                image.height() as i32,
                0,
                input_format,
                gl::UNSIGNED_BYTE,
                image.raw_pixels().as_ptr() as *const _,
            );
            gl::BindTexture(gl::TEXTURE_2D, 0);
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
