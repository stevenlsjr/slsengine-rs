use cgmath;
use sdl2::video::Window;
use std::time::{Duration, Instant};

type Vec2 = cgmath::Vector2<f32>;
type Vec3 = cgmath::Vector3<f32>;
type Vec4 = cgmath::Vector4<f32>;

type Mat3 = cgmath::Matrix3<f32>;
type Mat4 = cgmath::Matrix4<f32>;
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
    pos: Vec3,
    target: Vec3,
    direction: Vec3,
    up: Vec3,
    right: Vec3,
    transform: Mat4,
}

impl FpsCameraComponent {
    pub fn new() -> Self {
        use cgmath::{prelude::*, *};
        let direction = vec3(0.0, 0.0, 0.0);
        let right = vec3(0.0, 1.0, 0.0);
        let up = direction.cross(right);
        let transform = Mat4::identity();
        let mut cmp = FpsCameraComponent {
            pos: vec3(0.0, 0.0, -5.0),
            target: vec3(0.0, 0.0, 0.0),
            direction,
            up,
            right,
            transform,
        };

        cmp.build_transform();

        cmp
    }

    fn build_transform(&mut self) {
        use cgmath::*;
        self.transform =  Matrix4::from_translation(self.pos);
    }

    pub fn transform(&self)-> &Mat4 {
        &self.transform
    }
}

pub struct EntityWorld {
    pub main_camera: FpsCameraComponent,
}

impl EntityWorld {
    pub fn new() -> Self {
        let main_camera = FpsCameraComponent::new();
        EntityWorld { main_camera }
    }

    pub fn update(&mut self, window: &Window, delta: Duration) {}
}
