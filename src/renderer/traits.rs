use super::{camera::*, mesh::*};
use game;
use std::{cell::Ref, time::Duration};

/// A trait encapsulating the game's rendering capabilities
pub trait Renderer {
    /// The type parameter for the renderer's texture representation
    type Texture;

    fn clear(&self) {}
    fn camera(&self) -> Ref<Camera>;
    fn set_clear_color(&mut self, _color: Color) {}
    fn on_resize(&self, _size: (u32, u32)) {}
    fn on_update(&mut self, _delta_time: Duration, _world: &game::EntityWorld) {
    }

    /// Hints the renderer to recompile shaders, when convenient
    fn flag_shader_recompile(&self) {}
}

pub trait Resizable {
    fn on_resize(&mut self, size: (u32, u32));
}
