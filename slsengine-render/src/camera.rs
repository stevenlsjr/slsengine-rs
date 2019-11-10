use super::traits::*;
use cgmath::*;
/*
 *  Camera
 **/

pub fn default_perspective() -> PerspectiveFov<f32> {
    PerspectiveFov {
        fovy: Deg(45.0).into(),
        aspect: 1.0,
        near: 0.1,
        far: 1000.0,
    }
}

pub struct Camera {
    pub projection: Matrix4<f32>,
    perspective: PerspectiveFov<f32>,
}

use std::fmt;
impl fmt::Debug for Camera {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Camera")
    }
}

impl Camera {
    pub fn new(perspective: PerspectiveFov<f32>) -> Self {
        Camera {
            perspective,
            projection: perspective.into(),
        }
    }

    pub fn perspective(&self) -> PerspectiveFov<f32> {
        self.perspective
    }

    fn build_perspective(&mut self) {
        self.projection = self.perspective.into();
    }
}

