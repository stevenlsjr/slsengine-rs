extern crate sdl2;
extern crate slsengine;

extern crate gl;

use slsengine::renderer::*;
use slsengine::sdl_platform::*;
// #[test]
fn test_shader_compile() {
    let (platform, _) = platform_with_gl();
    let good_src = r#"
    out vec4 out_color;
    void main(){
        out_color = vec4(1.0, 0.0, 0.0, 1.0);
    }
    "#;

    let bad_src = r#"
    out vec4 out_color;
    void main(){
        out_color = vec4(1.0, 0.0, 0.0, 1.0);

    "#; // unmatched brackets

    let header = "#version 410\n";

    let raw_shader = unsafe { gl::CreateShader(gl::FRAGMENT_SHADER) };

    assert!(raw_shader == gl::TRUE as u32, "glCreateShader call is failing!");
    unsafe {
        gl::DeleteShader(raw_shader);
    }

    let good_shader =
        unsafe { compile_source(&[header, good_src], gl::FRAGMENT_SHADER) };
    assert!(
        good_shader.is_ok(),
        "shader compilation should be successful: {}",
        good_shader.unwrap_err()
    );
    let bad_shader =
        unsafe { compile_source(&[header, bad_src], gl::FRAGMENT_SHADER) };
    assert!(bad_shader.is_err());
    if let Err(e) = bad_shader {
        assert!(match e {
            ShaderError::CompileFailure { .. } => true,
            _ => false,
        })
    }
    unsafe {
        gl::DeleteShader(good_shader.unwrap());
    }
}

fn platform_with_gl() -> (Platform, sdl2::video::GLContext) {
    let plt = platform().build().unwrap();

    let gl_ctx = plt
        .window
        .gl_create_context()
        .unwrap_or_else(|e| panic!("could not create context! {}", e));
    gl::load_with(|name| {
        plt.video_subsystem.gl_get_proc_address(name) as *const _
    });

    plt.window.gl_set_context_to_current().unwrap();

    (plt, gl_ctx)
}

fn main() {
    use std::panic;
    use std::process::exit;

    test_shader_compile();
}
