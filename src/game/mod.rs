use super::math::*;
use cgmath::*;
use sdl2::{keyboard::KeyboardState, video::Window};
use std::time::{Duration, Instant};

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

pub struct FpsCameraComponent {
    pos: Point3<f32>,
    target: Vec3,
    direction: Vec3,
    up: Vec3,
    right: Vec3,
    transform: Mat4,
    speed: f32,
}

impl FpsCameraComponent {
    pub fn new() -> Self {
        use cgmath::{prelude::*, *};
        let direction = vec3(0.0, 0.0, -1.0);
        let right = vec3(0.0, 1.0, 0.0);
        let up = direction.cross(right);
        let transform = Mat4::identity();
        let mut cmp = FpsCameraComponent {
            pos: Point3::new(0.0, 0.0, -5.0),
            target: vec3(0.0, 0.0, 0.0),
            direction,
            up,
            right,
            transform,
            speed: 3.0,
        };

        cmp.build_transform();

        cmp
    }

    fn build_transform(&mut self) {
        use cgmath::*;
        self.transform = Matrix4::look_at(self.pos, Point3::new(0.0, 0.0, 0.0), self.right);
    }

    pub fn transform(&self) -> &Mat4 {
        &self.transform
    }
}

pub struct EntityWorld {
    pub main_camera: FpsCameraComponent,
}

pub struct InputState<'a> {
    pub keyboard_state: KeyboardState<'a>,
}

impl EntityWorld {
    pub fn new() -> Self {
        let main_camera = FpsCameraComponent::new();
        EntityWorld { main_camera }
    }

    pub fn update(&mut self, delta: Duration, input: InputState) {
        use sdl2::keyboard::Scancode;
        let mut wasd_axis = Vec2::new(0.0, 0.0);
        {
            let InputState { keyboard_state } = &input;

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
        }
        // println!("wasd_axis: {:?} {}", wasd_axis, wasd_axis.magnitude());
        if wasd_axis.magnitude() > 0.0 {
            self.move_camera(wasd_axis, delta.as_float_secs(), input);
            self.main_camera.build_transform();
        }
    }

    pub fn move_camera(&mut self, move_axis: Vec2, dt: f64, input: InputState) {
        use cgmath::prelude::*;
        let delta_pos = {
            let v2 = self.main_camera.speed * dt as f32 * move_axis;
            Vec3 { y: 0.0, ..v2.xyy() }
        };
        self.main_camera.pos += delta_pos;

    }
}
