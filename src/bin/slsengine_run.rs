extern crate genmesh;
extern crate gl;
extern crate sdl2;
extern crate slsengine;

use slsengine::renderer::gl_renderer::*;
use slsengine::*;

fn create_shaders() -> Result<Program, renderer::ShaderError> {
    use renderer::*;
    let header: &'static str = "#version 410\n";
    let vs = unsafe {
        compile_source(
            &[
                header,
                r#"
layout (location = 0) in vec3 v_pos;

void main(){
    gl_Position = vec4(v_pos, 1.0);
}
                "#,
            ],
            gl::VERTEX_SHADER,
        )
    }?;

    let fs = unsafe {
        compile_source(
            &[
                header,
                r#"

out vec4 out_color;
void main(){
    float r = sin(gl_FragCoord.x / 10.0) / 0.5 + 0.5;
    float g = sin(gl_FragCoord.y / 10.0) / 0.5 + 0.5;
    out_color = vec4(r, g, 1.0, 1.0);
}
            "#,
            ],
            gl::FRAGMENT_SHADER,
        )
    }?;

    ProgramBuilder::new()
        .frag_shader(fs.0)
        .vert_shader(vs.0)
        .build_program()
}

fn make_mesh() {
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
        }).triangulate()
    };

    let verts: Vec<SlsVertex> = generator().vertices().collect();
    let indices: Vec<_> = verts.iter().zip(0..).map(|(_, b)| b).collect();
    println!(
        "vertices: len {}, indices: len {}",
        verts.len(),
        indices.len()
    );
}

pub fn game_main() {
    use sdl_platform::{platform, OpenGLVersion, Platform};

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

    let _square_mesh = make_mesh();

    loop_state.is_running = true;

    while loop_state.is_running {
        loop_state.handle_events(&window, event_pump.borrow_mut().poll_iter());
        unsafe {
            gl::ClearColor(0.6, 0.0, 0.8, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        program.use_program();

        window.gl_swap_window();
    }
}

fn main() {
    game_main();
}
