#![allow(unused_imports)]
#![allow(unused_variables)]
#![cfg(feature = "with-vulkan")]
#![feature(test)]

#[macro_use]
extern crate failure;
extern crate test;

use ::log::*;
use cgmath::prelude::*;
use sdl2::sys as sdl_sys;
use sdl2::video::*;
use sdl2::*;
use slsengine::renderer::backend_vk::*;
use slsengine::renderer::Camera;
use slsengine::sdl_platform::*;
use std::{
    cell::{Ref, RefCell},
    ffi::{CStr, CString},
    os::raw::c_char,
    ptr,
    rc::Rc,
    sync::Arc,
};
use vulkano;
use vulkano::{
    buffer::*,
    command_buffer::*,
    descriptor::descriptor_set::*,
    device::Device,
    framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract},
    image::swapchain::SwapchainImage,
    impl_vertex,
    pipeline::*,
    single_pass_renderpass,
    swapchain::Swapchain,
    sync::GpuFuture,
};
use vulkano_shaders;

mod vs {
    vulkano_shaders::shader! {
    ty: "vertex",
        src: "
        #version 450

        layout(location = 0) in vec2 position;
        
        void main(){
            gl_Position = vec4(position, 0.0, 1.0);
        }
        "
    }
}

mod fs {
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

#[derive(Debug, Clone, Copy)]
struct SimpleVert {
    position: [f32; 2],
}
impl_vertex!(SimpleVert, position);

struct VulkanPlatformHooks;

fn create_renderpass<W>(
    device: Arc<Device>,
    swapchain: &Swapchain<W>,
) -> Result<Arc<dyn RenderPassAbstract + Send + Sync>, failure::Error> {
    let rp = single_pass_renderpass!(device,
    attachments: {
        color: {
            load: Clear,
            store: Store,
            format: swapchain.format(),
            samples: 1,
        }
    },
    pass: {
        color: [color],
        depth_stencil: {}

    })?;
    Ok(Arc::new(rp))
}

impl PlatformBuilderHooks for VulkanPlatformHooks {
    fn build_window(
        &self,
        platform_builder: &PlatformBuilder,
        video_subsystem: &VideoSubsystem,
    ) -> PlatformResult<Window> {
        let mut wb = make_window_builder(platform_builder, video_subsystem);
        wb.vulkan();
        wb.resizable();
        let window = wb.build().unwrap();
        Ok(window)
    }
}

fn main() {
    use env_logger;
    use rand::{distributions::uniform::*, *};
    use slsengine::game;
    use std::{
        sync::Arc,
        time::{Duration, Instant},
    };
    use test::black_box;

    env_logger::init();

    let platform = platform().build(&VulkanPlatformHooks).unwrap();

    let renderer = black_box(VulkanRenderer::new(&platform.window).unwrap());
    let VulkanRenderer {
        ref queues,
        ref device,
        ref instance,
        ref swapchain,
        ..
    } = renderer;
    let verts = [
        SimpleVert {
            position: [-0.5, -0.25],
        },
        SimpleVert {
            position: [0.0, 0.5],
        },
        SimpleVert {
            position: [0.25, -0.1],
        },
    ];
    let vertex_buffer = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::all(),
        verts.iter().cloned(),
    )
    .expect("failed to create vertex array buffer");
    let vs =
        vs::Shader::load(device.clone()).expect("failed to load vert shader");
    let fs =
        fs::Shader::load(device.clone()).expect("failed to load frag shader");
    let render_pass = create_renderpass(device.clone(), swapchain).unwrap();
    let pipeline = {
        use vulkano::{framebuffer::*, pipeline::*};
        Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<SimpleVert>()
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fs.main_entry_point(), ())
                .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                .build(device.clone())
                .unwrap(),
        )
    };
    let mut dynamic_state = DynamicState {
        line_width: None,
        viewports: None,
        scissors: None,
    };
    let mut framebuffers = setup_framebuffers(
        &renderer.swapchain_images,
        render_pass.clone(),
        &mut dynamic_state,
    )
    .unwrap();
}

fn setup_framebuffers(
    images: &[Arc<SdlSwapchainImage>],
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    dynamic_state: &mut DynamicState,
) -> Result<Vec<Arc<dyn FramebufferAbstract>>, failure::Error> {
    use vulkano::pipeline::viewport::Viewport;

    let dimensions = images[0].dimensions();
    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
        depth_range: 0.0..1.0,
    };
    dynamic_state.viewports = Some(vec![viewport]);
    let mut fbs: Vec<Arc<dyn FramebufferAbstract>> =
        Vec::with_capacity(images.len());
    for image in images {
        let fb = Framebuffer::start(render_pass.clone())
            .add(image.clone())
            .and_then(|fbb| fbb.build())
            .map(&Arc::new)?;
        fbs.push(fb as Arc<FramebufferAbstract>);
    }
    Ok(fbs)
}
