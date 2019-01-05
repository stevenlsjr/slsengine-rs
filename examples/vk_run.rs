use failure;
use image::{ImageBuffer, Rgba};
use slsengine::{self, renderer::backend_vk::*, sdl_platform::*};
use std::sync::Arc;
use vulkano::{
    buffer::*,
    command_buffer::*,
    descriptor::descriptor_set::*,
    format::*,
    framebuffer::*,
    image::*,
    impl_vertex,
    pipeline::{viewport::*, *},
    single_pass_renderpass,
    sync::*,
};
use vulkano_shaders;
mod fractal_comp {
    vulkano_shaders::shader! {
        path: "assets/shaders/vulkan/fractal.comp",
        ty: "compute"
    }
}

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
struct SimpleVertex {
    position: [f32; 2],
}
impl_vertex!(SimpleVertex, position);

fn triangle_verts() -> Vec<SimpleVertex> {
    let vertex1 = SimpleVertex {
        position: [-0.5, -0.5],
    };
    let vertex2 = SimpleVertex {
        position: [0.0, 0.5],
    };
    let vertex3 = SimpleVertex {
        position: [0.5, -0.25],
    };
    vec![vertex1, vertex2, vertex3]
}

fn main() {
    use std::path::*;
    env_logger::init();
    let (width, height) = (1024, 1024);

    let platform = platform().build(&VulkanPlatformHooks).unwrap();

    let r = VulkanRenderer::new(&platform.window).unwrap();
    let q = &r.queues;
    let VulkanRenderer {
        ref device,
        queues: ref q,
        ..
    } = r;

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

    let vs =
        vs::Shader::load(device.clone()).expect("failed to load vert shader");
    let fs =
        fs::Shader::load(device.clone()).expect("failed to load frag shader");

    let mut dynamic_state = DynamicState {
        viewports: Some(vec![Viewport {
            origin: [0.0, 0.0],
            dimensions: [width as f32, height as f32],
            depth_range: 0.0..1.0,
        }]),
        line_width: None,

        scissors: None,
    };
    let vertex_array = {
        CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            triangle_verts().into_iter(),
        )
        .unwrap()
    };

    let render_pass = single_pass_renderpass! (
    device.clone(),
    attachments: {
        out_color: {load: Clear, store: Store, format: Format::R8G8B8A8Unorm,
        samples: 1,}
    },
    pass: {color: [out_color],
    depth_stencil: {}}
    )
    .map(&Arc::new)
    .unwrap();

    let pipeline = GraphicsPipeline::start()
        .vertex_input_single_buffer::<SimpleVertex>()
        .vertex_shader(vs.main_entry_point(), ())
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs.main_entry_point(), ())
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        .build(device.clone())
        .map(&Arc::new)
        .unwrap();

    let framebuffer = {
        Framebuffer::start(render_pass.clone())
            .add(image.clone())
            .unwrap()
    }
    .build()
    .map(&Arc::new)
    .unwrap();

    let cb = AutoCommandBufferBuilder::primary_one_time_submit(
        device.clone(),
        q.graphics_queue.family(),
    )
    .map_err(&failure::Error::from)
    .and_then(|cb| {
        cb.begin_render_pass(
            framebuffer.clone(),
            false,
            vec![[0.0, 0.0, 1.0, 1.0].into()],
        )
        .map_err(&failure::Error::from)
    })
    .and_then(|cb| {
        cb.draw(
            pipeline.clone(),
            &dynamic_state,
            vertex_array.clone(),
            (),
            (),
        )
        .map_err(failure::Error::from)
    })
    .and_then(|cb| cb.end_render_pass().map_err(&failure::Error::from))
    .and_then(|cb| {
        cb.copy_image_to_buffer(image.clone(), img_buffer.clone())
            .map_err(&failure::Error::from)
    })
    .unwrap()
    .build()
    .unwrap();

    let finished = cb.execute(q.graphics_queue.clone()).unwrap();
    finished
        .then_signal_fence_and_flush()
        .unwrap()
        .wait(None)
        .unwrap();

    save_img_buffer(&img_buffer, (width, height));
}

fn save_img_buffer(buf: &CpuAccessibleBuffer<[u8]>, size: (u32, u32)) {
    use std::path::*;
    let content = buf.read().unwrap();
    let path = Path::new(env!("OUT_DIR")).join("vk_run.png");
    let (width, height) = size;
    let img = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, &content[..])
        .unwrap();
    img.save(&path).unwrap();
    println!("image saved to {}", path.to_string_lossy())
}
