extern crate cgmath;
extern crate genmesh;
extern crate gl;
extern crate gltf;
extern crate sdl2;
extern crate slsengine;
extern crate stb_image;

#[macro_use]
extern crate failure;

use cgmath::prelude::*;
use cgmath::*;
use slsengine::renderer::gl_renderer::*;
use slsengine::renderer::objects;
use slsengine::*;
use std::ptr::null;

// returns the [u, v] surface coordinates for a unit sphere.
fn uv_for_unit_sphere(pos: Vector3<f32>) -> [f32; 2] {
    use std::f32::consts::PI;
    let normal: Vector3<f32> = pos.normalize();
    let u = (normal.x.atan2(normal.z) / (2.0 * PI) + 0.5)
        .min(1.0)
        .max(0.0);
    let v = (normal.y * 0.5 + 0.5).min(1.0).max(0.0);

    [u, v]
}

fn make_mesh() -> Result<Mesh, failure::Error> {
    use genmesh::generators::*;
    use genmesh::*;
    use slsengine::renderer::Vertex as SlsVertex;

    let generator = || {
        MapToVertices::vertex(SphereUv::new(32, 32), |v: Vertex| {
            use std::default::Default;
            let mut vert: SlsVertex = Default::default();
            vert.position = v.pos.into();
            vert.normal = v.pos.into();
            // approximate texture coordinates by taking long/lat on unit sphere.
            // obvs doesn't work great for non-spheres
            vert.uv = uv_for_unit_sphere(vec3(
                vert.position[0],
                vert.position[1],
                vert.position[2],
            ));
            vert
        })
        .triangulate()
    };

    let verts: Vec<SlsVertex> = generator().vertices().collect();
    let indices: Vec<_> = verts.iter().zip(0..).map(|(_, b)| b).collect();
    println!(
        "vertices: len {}, indices: len {}",
        verts.len(),
        indices.len()
    );
    Ok(Mesh {
        vertices: verts,
        indices,
    })
}

fn make_texture() -> objects::TextureObjects {
    use stb_image::image;
    let img: image::Image<u8> = match image::load("assets/checker-map.png") {
        image::LoadResult::ImageU8(i) => i,
        _ => panic!("unsupported image format!"),
    };

    let textures =
        objects::TextureObjects::new(1).expect("could not create texture");

    let id = textures.ids()[0];
    let in_format = if img.depth == 3 { gl::RGB } else { gl::RGBA };
    unsafe {
        gl::BindTexture(gl::TEXTURE_2D, id);
        gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MIN_FILTER,
            gl::LINEAR_MIPMAP_LINEAR as i32,
        );
        gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MAG_FILTER,
            gl::LINEAR as i32,
        );
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as i32,
            img.width as i32,
            img.height as i32,
            0,
            in_format,
            gl::UNSIGNED_BYTE,
            img.data.as_ptr() as *const _,
        );
        gl::GenerateMipmap(gl::TEXTURE_2D)
    }

    textures
}


fn main() {
    use renderer::objects::MeshBuffers;
    use renderer::BindUniform;
    use sdl_platform::{platform, OpenGLVersion, Platform};
    use std::time::*;

    let (plt, gl_platform_builder) = platform()
        .with_window_size(640, 480)
        .with_window_title("Rust opengl demo")
        .with_opengl(OpenGLVersion::GL41)
        .build_gl()
        .unwrap();
    let _ctx = gl_platform_builder.gl_ctx();

    let Platform {
        window, event_pump, ..
    } = plt;
    let mut loop_state = MainLoopState::new();
    let mesh = make_mesh().unwrap();
    let renderer = GlRenderer::new(&window, mesh.clone()).unwrap();

    let camera_view: Matrix4<f32> =
        Matrix4::<f32>::from_translation(Vector3::new(0.0, 0.0, -10.0));

    
    let program = renderer.scene_program();

    let _texture = make_texture();

    let modelview_id = program.uniform_location("modelview").unwrap();

    let mut timer = game::Timer::new(Duration::from_millis(1000 / 50));
    let mut world = game::EntityWorld::new();

    loop_state.is_running = true;
    while loop_state.is_running {
        loop_state.handle_events(
            &window,
            event_pump.borrow_mut().poll_iter(),
            &renderer,
        );
        let game::Tick { delta, .. } = timer.tick();

        let ticks = Instant::now().duration_since(timer.start_instant());
        let theta = game::duration_as_f64(ticks);

        let modelview = camera_view * Matrix4::from_angle_x(Rad(theta as f32));
        {
            world.update(&window, delta);
        }
        renderer.clear();
        renderer.render_scene(&world);

        window.gl_swap_window();
    }
}
