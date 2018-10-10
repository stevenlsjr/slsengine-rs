extern crate webgl_generator;

use webgl_generator::*;
use std::env;
use std::fs::File;
use std::path::*;

fn main(){
    let dest = env::var("OUT_DIR").unwrap();
    let mut _webgl_file = File::create(&Path::new(&dest).join("webgl_stdweb.rs")).unwrap();
    let mut webgl2_file = File::create(&Path::new(&dest).join("webgl2_stdweb.rs")).unwrap();

    // Registry::new(Api::WebGl, Exts::ALL)
    //     .write_bindings(StdwebGenerator, &mut webgl_file)
    //     .unwrap();

    Registry::new(Api::WebGl2, Exts::ALL)
        .write_bindings(StdwebGenerator, &mut webgl2_file)
        .unwrap();
        

}