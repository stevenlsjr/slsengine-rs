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

///
/// Constructs the camera view matrix for the scene.
#[derive(Debug)]
pub struct FpsCameraComponent {
    pos: Point3<f32>,
    front: Vec3,
    up: Vec3,
    right: Vec3,
    world_up: Vec3,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
    speed: f32,
    mouse_sensitivity: f32,
    transform: Mat4,

}

impl FpsCameraComponent {
    pub fn new(position: Point3<f32>, up: Vec3, yaw: Rad<f32>, pitch: Rad<f32>) -> Self {
        use cgmath::{prelude::*, *};
        let world_up = up.clone();
        let zero = vec3(0.0, 0.0, 0.0);
        let mut cmp = FpsCameraComponent {
            pos: position,
            up,
            world_up,
            yaw, 
            pitch,
            speed: 3.0,
            mouse_sensitivity: 1.0,
            // other fields given default values
            transform: Mat4::identity(),
            front: zero.clone(),
            right: zero.clone()
        };

        cmp.update_vectors();
        cmp.build_transform();

        cmp
    }

    /// Set front, up, and right vectors to appropriate values
    fn update_vectors(&mut self){
        let Rad(yaw) = self.yaw;
        let Rad(pitch) = self.pitch;
        let front = vec3(
            yaw.cos() * pitch.cos(),
            pitch.sin(),
            yaw.sin() * pitch.cos()
        ).normalize();
        self.front = front;
        self.right = front.cross(self.world_up).normalize();
        self.up = self.right.cross(self.front).normalize();
    }

    fn build_transform(&mut self) {
        use cgmath::*;
        self.transform = Mat4::look_at_dir(self.pos, self.front, self.up);
    }

    pub fn transform(&self) -> &Mat4 {
        &self.transform
    }

    pub fn input_move(&mut self, wasd_axis: Vec2, dt: f64, input: &InputState){
        use cgmath::prelude::*;
        let move_direction = (wasd_axis.x * self.right + wasd_axis.y * self.front).normalize();
        let delta_position = move_direction * self.speed * dt as f32;
        self.pos += delta_position;
        self.update_vectors();
        self.build_transform();

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
        use std::f32::consts::PI;
        let main_camera = FpsCameraComponent::new(Point3::new(0.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0), Rad(PI/2.0), Rad(0.0));
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

            if keyboard_state.is_scancode_pressed(Scancode::Y){
                println!("Camera: {:?}", self.main_camera);
            }
        }
        // println!("wasd_axis: {:?} {}", wasd_axis, wasd_axis.magnitude());
        if wasd_axis.magnitude() > 0.0 {

            self.main_camera.input_move(wasd_axis, delta.as_float_secs(), &input);
        }
    }


}
