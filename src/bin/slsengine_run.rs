extern crate gl;
extern crate sdl2;
extern crate slsengine;
#[cfg(target_arch = "wasm32")]
#[macro_use]
extern crate stdweb;

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
    out_color = vec4(1.0, 0.0, 1.0, 1.0);
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
    use super::*;

    pub fn game_main() -> Result<(), String> {
        use renderer::compile_source;
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

        loop_state.is_running = true;

        while loop_state.is_running {
            loop_state
                .handle_events(&window, event_pump.borrow_mut().poll_iter());
            unsafe {
                gl::ClearColor(0.6, 0.0, 0.8, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
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
