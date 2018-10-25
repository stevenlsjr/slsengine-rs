extern crate cgmath;
extern crate genmesh;
extern crate gl;
extern crate sdl2;
extern crate slsengine;

use cgmath::prelude::*;
use cgmath::*;
use slsengine::renderer::gl_renderer::*;
use slsengine::*;
use std::ptr::null;

fn create_shaders() -> Result<Program, renderer::ShaderError> {
    use renderer::ShaderError;
    use renderer::*;
    use std::fs::File;
    use std::io::Read;
    let header: &'static str = "#version 410\n";
    let mut vs_source = String::new();
    let mut fs_source = String::new();

    {
        let mut vsf =
            File::open("./assets/flat-shading.vert").map_err(|e| {
                ShaderError::CompileFailure {
                    info_log: format!("Error opening source {}", e),
                }
            })?;
        let mut fsf =
            File::open("./assets/flat-shading.frag").map_err(|e| {
                ShaderError::CompileFailure {
                    info_log: format!("Error opening source {}", e),
                }
            })?;

        vsf.read_to_string(&mut vs_source).map_err(|_| {
            ShaderError::CompileFailure {
                info_log: "could not read vert shader".to_string(),
            }
        })?;
        fsf.read_to_string(&mut fs_source).map_err(|_| {
            ShaderError::CompileFailure {
                info_log: "could not read vert shader".to_string(),
            }
        })?;
    }

    let vs =
        unsafe { compile_source(&[header, &vs_source], gl::VERTEX_SHADER) }?;

    let fs =
        unsafe { compile_source(&[header, &fs_source], gl::FRAGMENT_SHADER) }?;

    ProgramBuilder::new()
        .frag_shader(fs.0)
        .vert_shader(vs.0)
        .build_program()
}

fn make_mesh() -> Mesh {
    use genmesh::generators::Cone;
    use genmesh::*;
    use slsengine::renderer::Vertex as SlsVertex;
    let generator = || {
        MapToVertices::vertex(Cone::new(8), |v: Vertex| {
            use std::default::Default;
            let mut vert: SlsVertex = Default::default();
            vert.position = v.pos.into();
            vert.normal = v.pos.into();
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
    Mesh {
        vertices: verts,
        indices,
    }
}

pub fn game_main() {
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

    let program = create_shaders().unwrap();

    let mesh = make_mesh();
    let buffers =
        MeshBuffers::new().expect("could not build gl objects for mesh");
    buffers
        .bind_mesh(&mesh)
        .expect("could not bind mesh to buffers");

    let projection_id = program.uniform_location("projection").unwrap_or(0);
    let modelview_id = program.uniform_location("modelview").unwrap();

    let projection = cgmath::Matrix4::from(PerspectiveFov {
        fovy: Deg(45.0).into(),
        aspect: 1.0,
        near: 0.001,
        far: 1000.0,
    });
    let mut modelview: Matrix4<f32> = Matrix4::identity();
    let mut timer = game::Timer::new(Duration::from_millis(1000 / 50));
    let translation = Matrix4::<f32>::from_translation(Vector3::new(0.0, 0.0, -10.0));

    loop_state.is_running = true;
    while loop_state.is_running {
        loop_state.handle_events(&window, event_pump.borrow_mut().poll_iter());
        let game::Tick { delta, .. } = timer.tick();

        let ticks = Instant::now().duration_since(timer.start_instant());
        let theta = game::duration_as_f64(ticks);

        modelview = translation * Matrix4::from_angle_x(Rad(theta as f32));

        unsafe {
            gl::ClearColor(0.6, 0.0, 0.8, 1.0);
            gl::Enable(gl::DEPTH_TEST);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        program.use_program();
        program.bind_uniform(modelview_id, &modelview);
        program.bind_uniform(projection_id, &projection);
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

fn main() {
    game_main();
}
