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
use vulkano::{self, impl_vertex};
use vulkano_shaders;
mod vs {
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

mod fs {
    vulkano_shaders::shader! {
    ty: "fragment",
    src: "#version 450

    layout(location = 0) out vec4 out_color;
    
    void main(){
        out_color = vec4(1.0, 1.0, 1.0, 1.0);
    }
    "
    }

}

mod comp_shader {
    vulkano_shaders::shader! {
    ty: "compute",
    src: "#version 450

layout(local_size_x=64, local_size_y = 1, local_size_z = 1) in;
layout (set=0, binding=0) buffer Data {
    uint data[];
} buf; 


void main() {
    uint idx = gl_GlobalInvocationID.x;
    buf.data[idx] *= 12;
}
        "
    }
}

struct VulkanPlatformHooks;

// static FRAG_SPIRV: &[u8] =
//     include_bytes!("../assets/shaders/spirv/brdf.frag.spv");

// static VERT_SPIRV: &[u8] =
//     include_bytes!("../assets/shaders/spirv/brdf.vert.spv");

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
    use vulkano::{
        command_buffer::*, descriptor::descriptor_set::*, pipeline::*,
        sync::GpuFuture,
    };

    env_logger::init();

    let platform = platform().build(&VulkanPlatformHooks).unwrap();

    let renderer = black_box(VulkanRenderer::new(&platform.window).unwrap());
    let ref device = renderer.device;
    use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
    let data: Vec<u32> = thread_rng()
        .sample_iter(&Uniform::from(0..100))
        .take(65537)
        .collect();

    let data_buffer = CpuAccessibleBuffer::from_data(
        device.clone(),
        BufferUsage::all(),
        data.clone(),
    )
    .unwrap();
    let queue = &renderer.queues.present_queue;
    let shader = comp_shader::Shader::load(device.clone())
        .expect("could not load shader");
    let compute_pipeline = Arc::new(
        ComputePipeline::new(device.clone(), &shader.main_entry_point(), &())
            .expect("failed to create compute pipeline"),
    );
    let set = Arc::new(
        PersistentDescriptorSet::start(compute_pipeline.clone(), 0)
            .add_buffer(data_buffer.clone())
            .unwrap()
            .build()
            .unwrap(),
    );

    let command_buffer =
        AutoCommandBufferBuilder::new(device.clone(), queue.family())
            .unwrap()
            .dispatch([265, 1, 1], compute_pipeline.clone(), set.clone(), ())
            .unwrap()
            .build()
            .unwrap();
    {
        let finished = command_buffer.execute(queue.clone()).unwrap();
        finished
            .then_signal_fence_and_flush()
            .unwrap()
            .wait(None)
            .unwrap();
        let content = data_buffer.read().unwrap();
        eprintln!("first item: {}", content[0]);
    }
}
