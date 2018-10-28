extern crate cgmath;
extern crate genmesh;
extern crate gl;
extern crate sdl2;
extern crate slsengine;

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
    let u = normal.x.atan2(normal.z) / (2.0 * PI) + 0.5;
    let v = normal.y * 0.5 + 0.5;
    [u, v]
}

fn make_mesh() -> Mesh {
    use genmesh::generators::*;
    use genmesh::*;
    use slsengine::renderer::Vertex as SlsVertex;
    let generator = || {
        MapToVertices::vertex(Cone::new(32), |v: Vertex| {
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
        }).triangulate()
    };

    let verts: Vec<SlsVertex> = generator().vertices().collect();
    let indices: Vec<_> = verts.iter().zip(0..).map(|(_, b)| b).collect();
    println!(
        "vertices: len {}, indices: len {}",
        verts.len(),
        indices.len()
    );
    Mesh {
        vertices: verts,
        indices,
    }
}

fn make_texture() -> objects::TextureObjects {
    use image;
    let dimg =
        image::open("assets/checker-map.png").expect("could not load image");
    unsafe {
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
    }
    let tex =
        objects::TextureObjects::new(1).expect("could not create texture");

    //    tex.bind_to_image(&dimg);

    tex
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

    let renderer = GlRenderer::new(&window, Deg(45.0).into()).unwrap();

    let camera_view: Matrix4<f32> =
        Matrix4::<f32>::from_translation(Vector3::new(0.0, 0.0, -10.0));

    let mesh = make_mesh();
    let buffers =
        MeshBuffers::new().expect("could not build gl objects for mesh");
    buffers
        .bind_mesh(&mesh)
        .expect("could not bind mesh to buffers");
    let program = renderer.scene_program();

    let texture = make_texture();

    let modelview_id = program.uniform_location("modelview").unwrap();

    let mut timer = game::Timer::new(Duration::from_millis(1000 / 50));

    loop_state.is_running = true;
    while loop_state.is_running {
        loop_state.handle_events(
            &window,
            event_pump.borrow_mut().poll_iter(),
            &renderer,
        );
        let game::Tick { delta: _delta, .. } = timer.tick();

        let ticks = Instant::now().duration_since(timer.start_instant());
        let theta = game::duration_as_f64(ticks);

        let modelview = camera_view * Matrix4::from_angle_x(Rad(theta as f32));

        unsafe {
            gl::ClearColor(0.6, 0.0, 0.8, 1.0);
            gl::Enable(gl::DEPTH_TEST);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        program.use_program();
        program.bind_uniform(modelview_id, &modelview);
        unsafe {
            gl::BindVertexArray(buffers.vertex_array.id());
            gl::DrawElements(
                gl::TRIANGLES,
                mesh.indices.len() as i32,
                gl::UNSIGNED_INT,
                null(),
            );
        }

        window.gl_swap_window();
    }
}
