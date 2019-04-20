use std::{cell::Ref, fmt, time::Duration};

use cgmath::*;
use sdl2::log::Category::Render;
use specs::prelude::*;
use specs::world::EntitiesRes;

use crate::game;
use crate::renderer::components::{MeshComponent, TransformComponent};

use super::{camera::*, mesh::*};
use super::color::*;
use sdl2::video::Window;

/// A trait encapsulating the game's rendering capabilities
pub trait Renderer: Sized {
    /// The type parameter for the renderer's texture representation

    /// Optional method. Callback for cleaning up resources, especially GPU resources
    /// sent to a free list rather than released immediately
    fn cleanup(&self) {}

    fn clear(&self) {}
    fn camera(&self) -> Ref<Camera>;
    fn set_clear_color(&mut self, _color: ColorRGBA) {}
    fn on_resize(&self, _size: (u32, u32)) {}

    //

    /// Hints the renderer to recompile shaders, when convenient
    fn flag_shader_recompile(&self) {}




    /// code dispatched by RenderSystem.
    fn render_system<'a>(&self, window: &Window, world: &mut World);
}



pub(crate)  type RenderSystemData<'a> = (Entities<'a>, ReadStorage<'a, MeshComponent>,
                                 ReadStorage<'a, TransformComponent>);



pub trait Resizable {
    fn on_resize(&mut self, size: (u32, u32));
}

/// trait for drawing text in screenspace
pub trait RenderText<S> {
    fn render_text(
        program: &S,
        text: &str,
        position: Point3<f32>,
        size: Vector3<f32>,
    );
}
