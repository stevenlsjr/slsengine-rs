use console_error_panic_hook;
use console_log;
use std::panic;
use wasm_bindgen::prelude::*;
use web_sys::HtmlElement;

use log::{debug, Level};
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
    platform::web::*
};

#[wasm_bindgen]
pub fn sample_main(root: HtmlElement) {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(Level::Debug);
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .build(&event_loop)
        .expect("could not build window");
    root.append_child(&window.canvas()).expect("could not set up canvas");
    debug!("hello!!!! {:?}\n{:?}", root, window.canvas());
    
    
}
