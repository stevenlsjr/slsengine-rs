use super::math::*;
use cgmath::*;
use sdl2::{keyboard::KeyboardState, mouse::MouseState, EventPump};
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub mod camera;
pub mod component;
pub use self::camera::*;

/*--------------------------------------
 * Game timer: handles delta time, time since start, etc
 */

/// Converts a duration to a floating point number.
/// Will perform unchecked math, so a very long duration will overflow
pub fn duration_as_f64(dur: Duration) -> f64 {
    let sec: u64 = dur.as_secs() * 1000;
    let milli: u64 = dur.subsec_millis() as u64;
    let result = (sec + milli) as f64 / 1000f64;
    result
}

#[test]
fn test_duration_as_f64() {
    let dur = Duration::from_secs(10);
    assert_eq!(duration_as_f64(dur), 10.0);
}

#[derive(Clone, Debug)]
pub struct Timer {
    last_instant: Instant,
    start_instant: Instant,
    pub desired_period: Duration,
}

#[derive(Clone, Debug)]
pub struct Tick {
    pub last_instant: Instant,
    pub delta: Duration,
}

impl Timer {
    pub fn new(desired_period: Duration) -> Timer {
        let start_instant = Instant::now();
        Timer {
            desired_period,
            start_instant,
            last_instant: start_instant,
        }
    }

    pub fn last_instant(&self) -> Instant {
        self.last_instant
    }

    pub fn start_instant(&self) -> Instant {
        self.start_instant
    }

    pub fn dur_from_start(&self, instant: Instant) -> Duration {
        self.start_instant.duration_since(instant)
    }

    pub fn tick(&mut self) -> Tick {
        let last_instant = self.last_instant;
        self.last_instant = Instant::now();
        let delta = self.last_instant.duration_since(last_instant);
        Tick {
            last_instant,
            delta,
        }
    }
}

/*--------------------------------------
 * Scene
 */

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

#[derive(Debug)]
pub struct EntityWorld {
    pub input_state: Option<InputState>,
    pub main_camera: FpsCameraComponent,
    pub components: component::ComponentManager,
}

impl EntityWorld {
    pub fn new() -> Self {
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
            components: component::ComponentManager::new(),
        };
        world.setup_game();
        world
    }

    fn setup_game(&mut self) {
        use self::component::{ComponentMask, TransformComponent};
        use renderer::material::*;
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

                let mut material = UntexturedMat {
                    roughness_factor: (j as f32) / ((n_rows - 1) as f32),
                    metallic_factor: (i as f32) / ((n_cols - 1) as f32),
                    ..base::PLASTIC_WHITE
                };
                material.roughness_factor = material.roughness_factor.max(0.01).min(1.0);
                material.metallic_factor = material.metallic_factor.max(0.01).min(1.0);

                eprintln!(
                    "material for {} {} with position {:?}: {:?}",
                    i,
                    j,
                    xform.transform.disp,
                    (material.roughness_factor, material.metallic_factor)
                );
                self.components.materials.insert(eid, material);
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
                println!("Camera: {:?}", self.main_camera);
            }
        }
        // println!("wasd_axis: {:?} {}", wasd_axis, wasd_axis.magnitude());
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
