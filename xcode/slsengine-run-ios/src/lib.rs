use sdl2;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
extern crate failure;

fn run_app() -> Result<(), failure::Error> {
    let sdl = sdl2::init().map_err(failure::err_msg)?;
    let video = sdl.video().map_err(failure::err_msg)?;
    let win = video
        .window("Ios App", 480, 640)
        .fullscreen_desktop()
        .opengl()
        .build()?;
    Ok(())
}

#[no_mangle]
pub extern "C" fn sls_app_start() -> c_int {
    match run_app() {
        Ok(_) => true as c_int,
        Err(e) => {
            eprintln!("fatal error: {:?}", e);
            1
        }
    }
}
