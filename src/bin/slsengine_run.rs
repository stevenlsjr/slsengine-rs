extern crate gl;
#[macro_use]
extern crate memoffset;
extern crate sdl2;
extern crate slsengine;
#[cfg(target_arch = "wasm32")]
#[macro_use]
extern crate stdweb;

use gl::types::*;
use slsengine::*;

#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::*;

    pub fn game_main() -> Result<(), String> {
        use sdl_platform::{platform, Platform};

        stdweb::initialize();

        let Platform {
            window,
            video_subsystem,
            event_pump,
            ..
        } = platform()
            .with_window_size(640, 480)
            .with_window_title("Rust opengl demo")
            .build()?;
        println!("window: {:?}", window.size());
        stdweb::event_loop();

        let _gl_ctx = window.gl_create_context()?;
        gl::load_with(|name| {
            video_subsystem.gl_get_proc_address(name) as *const _
        });
        window.gl_set_context_to_current()?;

        unsafe {
            gl::ClearColor(0.0, 1.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        window.gl_swap_window();

        Ok(())
    }
}

fn create_shaders() -> Result<(u32, u32), renderer::ShaderError> {
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
    Ok((vs, fs))
}

#[cfg(not(target_arch = "wasm32"))]
mod desktop {
    use renderer::objects::*;
    use slsengine::renderer::gl_renderer::*;
    use super::*;

    fn create_mesh_object(
        mesh: &Mesh,
        program: &Program,
    ) -> Result<MeshBuffers, String> {
        use gl::types::*;
        use std::ffi::c_void;
        use std::mem::size_of;

        let buffers = MeshBuffers::new().map_err(|e| format!("{:?}", e))?;

        program.use_program();
        let verts = [-1f32, -1f32, 0f32,
            -1f32, 1f32, 0f32,
            1f32, 1f32, 0f32,
            1f32, -1f32, 0f32];

        let vert_size = verts.len() * size_of::<f32>();
        let elements = [0u32, 1, 2, 2, 3, 0];
        let elements_size = elements.len() * size_of::<u32>();
        unsafe {
            buffers.vertex_array.bind();
            buffers.index_buffer.bind(gl::ELEMENT_ARRAY_BUFFER);
            buffers.vertex_buffer.bind(gl::ARRAY_BUFFER);


            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                elements_size as GLsizeiptr,
                elements.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );
            gl::BufferData(
                gl::ARRAY_BUFFER,
                vert_size as GLsizeiptr,
                verts.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                0,
                0 as *const c_void
            );

            gl::EnableVertexAttribArray(0);
        }

        Ok(buffers)
    }

    pub fn game_main() -> Result<(), String> {
        use renderer::ProgramBuilder;
        use sdl_platform::{platform, Platform};

        let Platform {
            window,
            video_subsystem,
            event_pump,
            ..
        } = platform()
            .with_window_size(640, 480)
            .with_window_title("Rust opengl demo")
            .build()?;
        let mut loop_state = MainLoopState::new();
        let _gl_ctx = window.gl_create_context()?;

        gl::load_with(|name| {
            video_subsystem.gl_get_proc_address(name) as *const _
        });
        window.gl_set_context_to_current()?;

        let (vs, fs) = create_shaders().unwrap();
        let mut pb = ProgramBuilder::new();
        pb.frag_shader(fs);
        pb.vert_shader(vs);

        let program = pb.build_program().unwrap();

        let square_mesh = renderer::rectangle_mesh().build().unwrap();
        let mesh_buffers = create_mesh_object(&square_mesh, &program).unwrap();

        unsafe {
            assert_eq!(gl::IsProgram(program.id), gl::TRUE);
            gl::DeleteShader(vs);
            gl::DeleteShader(fs);
        }

        loop_state.is_running = true;

        while loop_state.is_running {
            loop_state
                .handle_events(&window, event_pump.borrow_mut().poll_iter());
            unsafe {
                gl::ClearColor(0.6, 0.0, 0.8, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }

            program.use_program();
            let n_elements = square_mesh.indices.len() as GLsizei;
            let (width, height) = window.size();

            unsafe {
                gl::Viewport(0, 0, width as i32, height as i32);
                mesh_buffers.vertex_array.bind();
                gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0 as *const _);
            }

            window.gl_swap_window();
        }
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    if let Err(e) = desktop::game_main() {
        eprintln!("Game error: {}", e);
        std::process::exit(-1);
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {
    stdweb::initialize();
    if let Err(e) = wasm::game_main() {
        eprintln!("wasm failed: {}", e);
    }
}
