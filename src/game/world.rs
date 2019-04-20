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

use specs::shred::*;


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
    pub main_camera: FpsCameraComponent,
    world: World,
}

impl fmt::Debug for WorldManager
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::any::TypeId;
        f.debug_struct(&"EntityWorld<R>")

            .field("main_camera", &format_args!("{{..}}"))
            .finish()
    }
}



impl WorldManager
{
    pub fn new<R: Renderer>(_renderer: &R) -> Self {
        use crate::renderer::components::*;

        use std::f32::consts::PI;
        let main_camera = FpsCameraComponent::new(
            Point3::new(0.0, 0.0, 5.0),
            vec3(0.0, 1.0, 0.0),
            Rad(-PI / 2.0),
            Rad(0.0),
        );

        let mut world = World::new();
        world.register::<MeshComponent>();
        world.register::<TransformComponent>();
        // add base resources

        world.add_resource(DeltaTime(Duration::new(0, 0)));
        world.add_resource::<Option<InputState>>(None);
//        world.add_resource(main_camera);

        WorldManager {
            main_camera,
            world,
        }
    }


    pub fn read_input_state(&self) -> Fetch<Option<InputState>>{
        self.world.read_resource()
    }

    pub fn write_input_state(&mut self) ->FetchMut<Option<InputState>> {
        self.world.write_resource()
    }

    pub fn world(&self) -> &World {&self.world}
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }
}
