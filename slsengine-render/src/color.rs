use cgmath::*;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[repr(C)]
pub struct ColorRGBA {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

pub fn color4f(r: f32, g: f32, b: f32, a: f32) -> ColorRGBA {
    ColorRGBA { r, g, b, a }
}

impl Into<Vector4<f32>> for ColorRGBA {
    fn into(self) -> Vector4<f32> {
        ::cgmath::vec4(self.r, self.g, self.b, self.a)
    }
}

impl From<Vector4<f32>> for ColorRGBA {
    fn from(v: Vector4<f32>) -> Self {
        ColorRGBA {
            r: v.x,
            g: v.y,
            b: v.z,
            a: v.w,
        }
    }
}
