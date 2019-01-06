use super::VkContextError;
use super::*;
use std::sync::Arc;
use vulkano::{device::*, framebuffer::*, pipeline::*};

mod main_vs {
    vulkano_shaders::shader! {
    ty: "vertex",
        src: "
        #version 450

        layout(location = 0) in vec3 position;

        
        
        void main(){
            gl_Position = vec4(position, 1.0);
        }
        "
    }
}

mod main_fs {
    vulkano_shaders::shader! {
    ty: "fragment",
    src: "#version 450

    layout(location = 0) out vec4 out_color;
    
    void main(){
        out_color = vec4(1.0, 1.0, 0.0, 1.0);
    }
    "
    }

}
/// Contains primary pipelines used by application
#[derive(Clone)]
pub struct RendererPipelines {
    main_pipeline: DynGraphicsPipeline,
}
impl RendererPipelines {
    fn new_internal(
        device: &Arc<Device>,
        render_pass: &Arc<dyn RenderPassAbstract + Send + Sync>,
    ) -> Result<Self, failure::Error> {
        let vs = main_vs::Shader::load(device.clone())?;
        let fs = main_fs::Shader::load(device.clone())?;
        let subpass = match Subpass::from(render_pass.clone(), 0) {
            Some(sp) => sp,
            None => bail!("could not find render pass subpass"),
        };

        let main_pipeline = {
            let main = GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex>()
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fs.main_entry_point(), ())
                .render_pass(subpass)
                .build(device.clone())?;
            Arc::new(main)
        };
        Ok(RendererPipelines { main_pipeline })
    }
    pub fn new(
        device: &Arc<Device>,
        render_pass: &Arc<dyn RenderPassAbstract + Send + Sync>,
    ) -> Result<Self, VkContextError> {
        Self::new_internal(device, render_pass).map_err(|e| {
            VkContextError::component_creation("pipelines", Some(e))
        })
    }
}
