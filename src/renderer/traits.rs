use super::color::*;
use super::{camera::*, mesh::*};
use crate::game;
use cgmath::*;
use std::{cell::Ref, fmt, time::Duration};

/// A trait encapsulating the game's rendering capabilities
pub trait Renderer: Sized {
    /// The type parameter for the renderer's texture representation
    type Texture: Clone + fmt::Debug;
    type Mesh: RenderMesh + fmt::Debug;

    /// Optional method. Callback for cleaning up resources, especially GPU resources
    /// sent to a free list rather than released immediately
    fn cleanup(&self) {}

    fn clear(&self) {}
    fn camera(&self) -> Ref<Camera>;
    fn set_clear_color(&mut self, _color: ColorRGBA) {}
    fn on_resize(&self, _size: (u32, u32)) {}
    fn on_update(
        &mut self,
        _delta_time: Duration,
        _world: &game::EntityWorld<Self>,
    ) {
    }
    fn render_scene(&self, _scene: &game::EntityWorld<Self>) {}

    /// Hints the renderer to recompile shaders, when convenient
    fn flag_shader_recompile(&self) {}
}

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
