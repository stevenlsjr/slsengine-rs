use super::{camera::*, component::*};
use crate::math::*;
use crate::renderer::*;
use cgmath::*;
use log::*;
use sdl2::{keyboard::KeyboardState, mouse::MouseState, EventPump};
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Clone, Debug, PartialEq)]
pub struct InputState {
    pub last_mousepos: Point2<f32>,
    pub mousepos: Point2<f32>,
}

pub struct InputSources<'a> {
    pub keyboard_state: KeyboardState<'a>,
    pub mouse_state: MouseState,
}

impl<'a> InputSources<'a> {
    pub fn from_event_pump(event_pump: &'a EventPump) -> Self {
        InputSources {
            keyboard_state: event_pump.keyboard_state(),
            mouse_state: event_pump.mouse_state(),
        }
    }
}

pub struct EntityWorld<R>
where
    R: Renderer,
{
    pub input_state: Option<InputState>,
    pub main_camera: FpsCameraComponent,
    pub components: ComponentManager<R>,
}

impl<R> EntityWorld<R>
where
    R: Renderer,
{
    pub fn new(_renderer: &R) -> Self {
        use rand::random;
        use std::f32::consts::PI;
        let main_camera = FpsCameraComponent::new(
            Point3::new(0.0, 0.0, 5.0),
            vec3(0.0, 1.0, 0.0),
            Rad(-PI / 2.0),
            Rad(0.0),
        );

        let mut world = EntityWorld {
            main_camera,
            input_state: None,
            components: ComponentManager::new(),
        };
        world.setup_game();
        world
    }

    fn setup_game(&mut self) {
        use crate::renderer::material::*;
        let spacing = 3.0;
        /// setup grid of drawable entities
        let n_rows = 5;
        let n_cols = 5;
        for j in 0..n_rows {
            for i in 0..n_cols {
                use std::f32::consts::PI;
                let eid = self.components.alloc_entity();
                let mask = ComponentMask::TRANSFORM
                    | ComponentMask::STATIC_MESH
                    | ComponentMask::MATERIAL;
                self.components.masks[eid.0] =
                    self.components.masks[eid.0] | mask;
                let mut xform = TransformComponent {
                    ..TransformComponent::default()
                };
                let rotation: Euler<Rad<f32>> =
                    Euler::new(Rad(PI / 2.0), Rad::zero(), Rad::zero());
                xform.transform.disp = vec3(
                    (i as f32 - (n_cols as f32 / 2.0)) * spacing,
                    (j as f32 - (n_rows as f32 / 2.0)) as f32 * spacing,
                    -3.0,
                );
                xform.transform.rot = rotation.into();

                self.components.transforms.insert(eid, xform);
            }
        }
    }

    pub fn update(&mut self, delta: Duration, input: InputSources) {
        use sdl2::keyboard::Scancode;
        let input_state = self
            .input_state
            .clone()
            .expect("Event loop should have already populated input_state");
        let mouse_offset = {
            let mut m = input_state.mousepos - input_state.last_mousepos;
            m.y *= -1.0;
            m
        };
        let mut wasd_axis = Vec2::new(0.0, 0.0);
        {
            let InputSources { keyboard_state, .. } = &input;

            if keyboard_state.is_scancode_pressed(Scancode::W) {
                wasd_axis.y += 1.0;
            }
            if keyboard_state.is_scancode_pressed(Scancode::S) {
                wasd_axis.y -= 1.0;
            }
            if keyboard_state.is_scancode_pressed(Scancode::D) {
                wasd_axis.x += 1.0;
            }
            if keyboard_state.is_scancode_pressed(Scancode::A) {
                wasd_axis.x -= 1.0;
            }

            if keyboard_state.is_scancode_pressed(Scancode::Y) {
                info!("Camera: {:?}", self.main_camera);
            }
        }
        if wasd_axis.magnitude() > 0.0 {
            self.main_camera.input_move(
                wasd_axis,
                delta.as_float_secs(),
                &input,
            );
        }

        if mouse_offset.magnitude() > 0.0 && input.mouse_state.left() {
            self.main_camera
                .mouselook(mouse_offset, delta.as_float_secs());
        }
        if let Some(mut input_state) = self.input_state.clone() {
            input_state.last_mousepos = input_state.mousepos;
            self.input_state = Some(input_state);
        }
    }
}
