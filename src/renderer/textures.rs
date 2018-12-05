use super::gl_renderer::*;
use super::objects::*;
use gl;
use std::path::Path;

pub fn load_cubemap<P: AsRef<Path>>(
    up: P,
    down: P,
    right: P,
    left: P,
    front: P,
    back: P,
) {
    
}
