// --targets x86_64-apple-ios,aarch64-apple-ios
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
use gfx_backend_empty as back;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
use gfx_backend_metal as back;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

use failure::{format_err, Error, ResultExt};
use objc::class;
use objc::declare::ClassDecl;

fn class_decls() -> Result<(), Error> {
    let NSObject = class!(NSObject);
    let SlsTestObj = ClassDecl::new("SlsTestObj", NSObject)
        .ok_or(format_err!("could not declare SlsTestObj"));
    Ok(())
}

fn main() -> Result<(), i32> {
    class_decls().map_err(|e| {
        eprintln!("runtime error: {:?}", e);
        1
    })?;
    Ok(())
}

/**
 * ios entrypoint for renderer example
 **/
#[no_mangle]
pub extern "C" fn renderer_ios_run() -> i32 {
    match main() {
        Ok(_) => 0,
        Err(e) => e,
    }
}
