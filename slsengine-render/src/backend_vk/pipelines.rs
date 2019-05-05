use super::VkContextError;
use super::*;
use crate::::Vertex as SlsVertex;
use cgmath::*;
use failure::{self, bail};
use std::sync::Arc;
use vulkano::{
    descriptor::descriptor_set::FixedSizeDescriptorSetsPool, device::*,
    framebuffer::*, pipeline::*,
};

pub mod main_vs {
    vulkano_shaders::shader! {
    ty: "vertex",
    path: "assets/shaders/vulkan/flat.vert",
    }
}

mod main_fs {
    vulkano_shaders::shader! {
    ty: "fragment",
    path: "assets/shaders/vulkan/flat.frag",

    }
}

unsafe impl vertex::Vertex for SlsVertex {
    fn member(name: &str) -> Option<vertex::VertexMemberInfo> {
        use self::vertex::{VertexMemberInfo, VertexMemberTy};
        match name {
            "v_position" => Some(VertexMemberInfo {
                offset: offset_of!(SlsVertex, position),
                ty: VertexMemberTy::F32,
                array_size: 3,
            }),
            "v_normal" => Some(VertexMemberInfo {
                offset: offset_of!(SlsVertex, normal),
                ty: VertexMemberTy::F32,
                array_size: 3,
            }),
            "v_uv" => Some(VertexMemberInfo {
                offset: offset_of!(SlsVertex, uv),
                ty: VertexMemberTy::F32,
                array_size: 2,
            }),
            "v_tangent" => Some(VertexMemberInfo {
                offset: offset_of!(SlsVertex, tangent),
                ty: VertexMemberTy::F32,
                array_size: 3,
            }),
            "v_bitangent" => Some(VertexMemberInfo {
                offset: offset_of!(SlsVertex, bitangent),
                ty: VertexMemberTy::F32,
                array_size: 3,
            }),
            _ => None,
        }
    }
}

/// Contains primary pipelines used by application
pub struct RendererPipelines {
    pub main_pipeline: DynGraphicsPipeline,
    pub matrix_ubo: CpuBufferPool<main_vs::ty::MatrixData>,
    pub matrix_desc_pool:
        RwLock<FixedSizeDescriptorSetsPool<DynGraphicsPipeline>>,
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

        let main_pipeline: Arc<GraphicsPipelineAbstract + Send + Sync> =
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex>()
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fs.main_entry_point(), ())
                .depth_stencil_simple_depth()
                .front_face_clockwise()
                .cull_mode_back()
                // .polygon_mode_line()
                .render_pass(subpass)
                .build(device.clone())
                .map(&Arc::new)?;

        let matrix_desc_pool = RwLock::new(FixedSizeDescriptorSetsPool::new(
            main_pipeline.clone(),
            0,
        ));

        Ok(RendererPipelines {
            main_pipeline,
            matrix_ubo,
            matrix_desc_pool,
        })
    }
    pub fn new(
        device: &Arc<Device>,
        render_pass: &Arc<dyn RenderPassAbstract + Send + Sync>,
    ) -> Result<Self, VkContextError> {
        let matrix_ubo = CpuBufferPool::new(device.clone(), BufferUsage::all());
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
    pub fn new(
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

    pub fn modelview(&self) -> &Matrix4<f32> {
        &self.modelview
    }
    pub fn normal(&self) -> &Matrix4<f32> {
        &self.normal
    }
    pub fn projection(&self) -> &Matrix4<f32> {
        &self.projection
    }

    pub fn set_modelview(
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

impl From<MatrixUniformData> for main_vs::ty::MatrixData {
    fn from(data: MatrixUniformData) -> Self {
        main_vs::ty::MatrixData {
            modelview: data.modelview.into(),
            projection: data.projection.into(),
            normal: data.normal.into(),
        }
    }
}
