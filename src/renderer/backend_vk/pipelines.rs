use super::VkContextError;
use super::*;
use cgmath::*;
use failure::{self, bail};
use std::sync::Arc;
use vulkano::{device::*, framebuffer::*, pipeline::*};

pub mod main_vs {
    vulkano_shaders::shader! {
    ty: "vertex",
        src: "
        #version 450

        layout(location = 0) in vec3 position;
        layout (set=0, binding=0) uniform MatrixData {
            mat4 modelview;
            mat4 projection;
            mat4 normal;
        } m;
        
        
        void main(){
            gl_Position = m.projection * m.modelview * vec4(position, 1.0);
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
    pub main_pipeline: DynGraphicsPipeline,
    pub matrix_ubo: CpuBufferPool<main_vs::ty::MatrixData>,
}
impl RendererPipelines {
    fn new_internal(
        device: &Arc<Device>,
        render_pass: &Arc<dyn RenderPassAbstract + Send + Sync>,
        matrix_ubo: CpuBufferPool<main_vs::ty::MatrixData>,
    ) -> Result<Self, failure::Error> {
        let vs = main_vs::Shader::load(device.clone())?;
        let fs = main_fs::Shader::load(device.clone())?;
        let subpass = match Subpass::from(render_pass.clone(), 0) {
            Some(sp) => sp,
            None => bail!("could not find render pass subpass"),
        };

        let main_pipeline: Arc<GraphicsPipelineAbstract + Send + Sync> = {
            let main = GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex>()
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fs.main_entry_point(), ())
                .depth_stencil_simple_depth()
                .render_pass(subpass)
                .build(device.clone())
                .map(&Arc::new)?;
            main
        };

        
        // let main_matrix_set =
        //     PersistentDescriptorSet::start(main_pipeline.clone(), 0)
        //         .add_buffer(matrix_uniform_buffer)
        //         .unwrap()
        //         .build()
        //         .map_err(failure::Error::from);

        Ok(RendererPipelines {
            main_pipeline,
            matrix_ubo,
        })
    }
    pub fn new(
        device: &Arc<Device>,
        render_pass: &Arc<dyn RenderPassAbstract + Send + Sync>,
        matrix_ubo: CpuBufferPool<main_vs::ty::MatrixData>,
    ) -> Result<Self, VkContextError> {
        Self::new_internal(device, render_pass, matrix_ubo).map_err(|e| {
            VkContextError::component_creation("pipelines", Some(e))
        })
    }
}

pub struct MatrixUniformData {
    modelview: Matrix4<f32>,
    pub projection: Matrix4<f32>,
    normal: Matrix4<f32>,
}

impl MatrixUniformData {
    fn new(
        modelview: Matrix4<f32>,
        projection: Matrix4<f32>,
    ) -> Result<Self, failure::Error> {
        let mut data = MatrixUniformData {
            projection,
            normal: Matrix4::identity(),
            modelview: Matrix4::identity(),
        };

        match data.set_modelview(modelview) {
            Ok(_) => Ok(data),
            Err(e) => Err(e),
        }
    }

    fn modelview(&self) -> &Matrix4<f32> {
        &self.modelview
    }
    fn normal(&self) -> &Matrix4<f32> {
        &self.normal
    }
    fn projection(&self) -> &Matrix4<f32> {
        &self.projection
    }

    fn set_modelview(
        &mut self,
        modelview: Matrix4<f32>,
    ) -> Result<(), failure::Error> {
        match modelview.invert().map(|im| im.transpose()) {
            Some(normal) => {
                self.modelview = modelview;
                self.normal = normal;
                Ok(())
            }
            None => bail!("modelview matrix is not inversible"),
        }
    }
}