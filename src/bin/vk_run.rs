
extern crate slsengine;
use slsengine::sdl_platform::*;

fn main(){
    let plt = platform().with_window_size(640, 480)
        .with_window_title("Hello rust!")
        .with_vulkan()
        .build().unwrap();



}