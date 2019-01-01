use image::{ImageBuffer, Rgba};
use slsengine::{self, renderer::backend_vk::*, sdl_platform::*};
use vulkano::{buffer::*, command_buffer::*, format::*, image::*, sync::*};
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
    let cmd_buffer = {
        let mut builder = AutoCommandBufferBuilder::new(
            r.device.clone(),
            q.graphics_queue.family(),
        )
        .unwrap();
        builder = builder
            .clear_color_image(
                image.clone(),
                ClearValue::Float([1.0, 0.0, 1.0, 1.0]),
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
        img.save(&path);
        println!("saved image at {}", path.to_string_lossy());
    }
}
