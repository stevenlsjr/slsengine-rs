use super::*;
use crate::math::*;
use cgmath::*;
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
    pub fn new(
        position: Point3<f32>,
        up: Vec3,
        yaw: Rad<f32>,
        pitch: Rad<f32>,
    ) -> Self {
        use cgmath::{prelude::*, *};
        let world_up = up.clone();
        let zero = vec3(0.0, 0.0, 0.0);
        let mut cmp = FpsCameraComponent {
            pos: position,
            up,
            world_up,
            yaw,
            pitch,
            speed: 9.0,
            mouse_sensitivity: 0.1,
            // other fields given default values
            transform: Mat4::identity(),
            front: zero.clone(),
            right: zero.clone(),
        };

        cmp.update_vectors();
        cmp.build_transform();

        cmp
    }

    /// Set front, up, and right vectors to appropriate values
    fn update_vectors(&mut self) {
        let Rad(yaw) = self.yaw;
        let Rad(pitch) = self.pitch;
        let front = vec3(
            yaw.cos() * pitch.cos(),
            pitch.sin(),
            yaw.sin() * pitch.cos(),
        )
        .normalize();
        self.front = front;
        self.right = front.cross(self.world_up).normalize();
        self.up = self.right.cross(self.front).normalize();
    }

    fn build_transform(&mut self) {
        self.transform = Mat4::look_at_dir(self.pos, self.front, self.up);
    }

    pub fn transform(&self) -> &Mat4 {
        &self.transform
    }

    pub fn input_move(
        &mut self,
        wasd_axis: Vec2,
        dt: f64,
        _input: &InputSources,
    ) {
        use cgmath::prelude::*;
        let move_direction =
            (wasd_axis.x * self.right + wasd_axis.y * self.front).normalize();
        let delta_position = move_direction * self.speed * dt as f32;
        self.pos += delta_position;
        self.update_vectors();
        self.build_transform();
    }

    pub fn mouselook(&mut self, mouse_offset: Vec2, dt: f64) {
        use cgmath::*;
        let mut mouse_offset = mouse_offset;
        mouse_offset *= self.mouse_sensitivity * dt as f32;
        self.yaw += Rad(mouse_offset.x);
        self.pitch += Rad(mouse_offset.y);
        self.pitch = if self.pitch < Deg(-89.0).into() {
            Deg(-89.0).into()
        } else if self.pitch > Deg(89.0).into() {
            Deg(89.0).into()
        } else {
            self.pitch
        };
        self.update_vectors();
        self.build_transform();
    }
}
