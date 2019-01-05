use failure;
use image::{ImageBuffer, Rgba};
use slsengine::{self, renderer::backend_vk::*, sdl_platform::*};
use std::sync::Arc;
use vulkano::{
    buffer::*, command_buffer::*, descriptor::descriptor_set::*, format::*,
    image::*, pipeline::*, sync::*,
};
use vulkano_shaders;
mod fractal_comp {
    vulkano_shaders::shader! {
        path: "assets/shaders/vulkan/fractal.comp",
        ty: "compute"
    }
}

fn main() {
    use std::path::*;
    env_logger::init();
    let (width, height) = (1024, 1024);

    let platform = platform().build(&VulkanPlatformHooks).unwrap();

    let r = VulkanRenderer::new(&platform.window).unwrap();
    let q = &r.queues;

    let comp_pipeline = {
        let shader = fractal_comp::Shader::load(r.device.clone())
            .expect("could not load shader");
        Arc::new(
            ComputePipeline::new(
                r.device.clone(),
                &shader.main_entry_point(),
                &(),
            )
            .expect("could not load pipeline"),
        )
    };

    let image = StorageImage::new(
        r.device.clone(),
        Dimensions::Dim2d { width, height },
        Format::R8G8B8A8Unorm,
        Some(q.graphics_queue.family()),
    )
    .unwrap();

    let img_buffer = CpuAccessibleBuffer::from_iter(
        r.device.clone(),
        BufferUsage::all(),
        (0..width * height * 4).map(|_| 0u8),
    )
    .expect("failed to create buffer for image storage");
    let descriptor_set = Arc::new(
        (PersistentDescriptorSet::start(comp_pipeline.clone(), 0)
            .add_image(image.clone())
            .unwrap())
        .build()
        .unwrap(),
    );
    let cmd_buffer = {
        let mut builder = AutoCommandBufferBuilder::new(
            r.device.clone(),
            q.graphics_queue.family(),
        )
        .unwrap();
        builder = builder
            .dispatch(
                [width / 8, height / 8, 1],
                comp_pipeline.clone(),
                descriptor_set.clone(),
                (),
            )
            .unwrap();
        builder = builder
            .copy_image_to_buffer(image.clone(), img_buffer.clone())
            .unwrap();
        builder.build().unwrap()
    };
    let finished = cmd_buffer.execute(q.graphics_queue.clone()).unwrap();
    finished
        .then_signal_fence_and_flush()
        .unwrap()
        .wait(None)
        .unwrap();

    {
        let buffer_contents = img_buffer.read().unwrap();
        let data = &buffer_contents[..];
        let img =
            ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, data).unwrap();
        let path = Path::new(env!("OUT_DIR")).join("image.png");
        img.save(&path).unwrap();
        println!("saved image at {}", path.to_string_lossy());
    }
}
