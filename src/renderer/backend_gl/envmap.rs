// utilities for environment maps
use super::program::*;

#[derive(Clone, Copy, Debug)]
pub struct EnvmapUniforms {
    pub projection: Option<u32>,
    pub modelview: Option<u32>,
    pub cubemap_tex: Option<u32>
}

impl ShaderUniforms for EnvmapUniforms {
    fn find_locations(&mut self, program: &Program<Self>) {
        let mut uniforms = [
            (&mut self.projection, "projection"),
            (&mut self.modelview, "modelview"),
            (&mut self.cubemap_tex, "cubemap_tex"),
        ];

        for (ref mut ptr, name) in &mut uniforms {
            **ptr = program.uniform_location(name).unwrap_or(None);
        }
    }
}