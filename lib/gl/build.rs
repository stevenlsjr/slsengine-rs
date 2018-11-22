
extern crate gl_generator;

use gl_generator::*;
use std::env;
use std::fs::File;
use std::path::Path;





fn main(){
    let dest = env::var("OUT_DIR").unwrap();
    let target = env::var("TARGET").unwrap();
    let triple: Vec<_> = target.split("-").collect();
    let (arch, _os, platform) = (triple[0], triple[1], triple[2]);
    let mut file = File::create(&Path::new(&dest).join("gl_bindings.rs")).unwrap();
    if arch == "wasm32" {
        let extensions = ["GL_KHR_debug", "GL_ARB_debug_outupt"];

        if platform != "emscripten"{
            panic!("native gl/es generator does not support webassembly");
        }
        Registry::new(Api::Gles2, (3, 2), Profile::Core, Fallbacks::All, extensions)
            .write_bindings(GlobalGenerator, &mut file)
            .unwrap();
    } else {
        let extensions = ["GL_KHR_debug", "GL_ARB_debug_outupt"];

        Registry::new(Api::Gl, (4, 5), Profile::Core, Fallbacks::All, extensions)
            .write_bindings(GlobalGenerator, &mut file)
            .unwrap();
    }
    
}