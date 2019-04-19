use super::camera::*;
use crate::game::resource::DeltaTime;
use crate::math::*;
use crate::renderer::*;
use cgmath::*;
use log::*;
use sdl2::{keyboard::KeyboardState, mouse::MouseState, EventPump};
use specs::prelude::*;
use std::fmt;
use std::marker::PhantomData;
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

pub struct WorldManager
where
{
    pub input_state: Option<InputState>,
    pub main_camera: FpsCameraComponent,
    world: World,
}

impl fmt::Debug for WorldManager
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
            .finish()
    }
}

impl WorldManager
{
    pub fn new<R: Renderer>(_renderer: &R) -> Self {
        use std::f32::consts::PI;
        let main_camera = FpsCameraComponent::new(
            Point3::new(0.0, 0.0, 5.0),
            vec3(0.0, 1.0, 0.0),
            Rad(-PI / 2.0),
            Rad(0.0),
        );

        let mut world = World::new();
        // add base resources
        world.add_resource(DeltaTime(Duration::new(0, 0)));

        WorldManager {
            main_camera,
            input_state: None,
            world,
        }
    }

    pub fn world(&self) -> &World {&self.world}
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }
}
