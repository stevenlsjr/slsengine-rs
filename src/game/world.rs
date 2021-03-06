

use super::{camera::*, component::*, TryGetComponent, resource::ResourceManager};
use crate::math::*;
use crate::renderer::*;
use cgmath::*;
use log::*;
use sdl2::{keyboard::KeyboardState, mouse::MouseState, EventPump};
use std::fmt;
use std::time::Duration;

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

pub struct EntityWorld<R, CS>
where
    R: Renderer,
    CS: TryGetComponent
{
    pub input_state: Option<InputState>,
    pub main_camera: FpsCameraComponent,
    pub components: ComponentManager<CS>,
    pub resources: ResourceManager<R>,
}

impl<R, CS> fmt::Debug for EntityWorld<R, CS>
where
    R: Renderer,
    CS: TryGetComponent

{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::any::TypeId;
        f.debug_struct(&"EntityWorld<R>")
            .field(
                "input_state",
                &format_args!(
                    "{}",
                    if self.input_state.is_some() {
                        "Some({{..}}"
                    } else {
                        "None"
                    }
                ),
            )
            .field("main_camera", &format_args!("{{..}}"))
            .field("components", &format_args!("{{..}}"))
            .field("resources", &format_args!("{{..}}"))
            .finish()
    }
}

impl<R, CS> EntityWorld<R, CS>
where
    R: Renderer,
    CS: TryGetComponent

{
    pub fn new(_renderer: &R, component_store: CS) -> Self {
        use std::f32::consts::PI;
        let main_camera = FpsCameraComponent::new(
            Point3::new(0.0, 0.0, 5.0),
            vec3(0.0, 1.0, 0.0),
            Rad(-PI / 2.0),
            Rad(0.0),
        );

        EntityWorld {
            main_camera,
            input_state: None,
            components: ComponentManager::new(component_store),
            resources: ResourceManager::new(),
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
                delta.as_millis() as f64 / 1000.0,
                &input,
            );
        }

        if mouse_offset.magnitude() > 0.0 && input.mouse_state.left() {
            self.main_camera
                .mouselook(mouse_offset, delta.as_millis() as f64 / 1000.0);
        }
        if let Some(mut input_state) = self.input_state.clone() {
            input_state.last_mousepos = input_state.mousepos;
            self.input_state = Some(input_state);
        }
    }
}
