#[cfg(target_arch = "wasm32")]
mod webgl {
    use super::*;
    extern crate stdweb;

    use std::default::Default;

    // Shamelessly stolen from webplatform's TodoMVC example.
    macro_rules! enclose {
    ( ($( $x:ident ),*) $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

    #[cfg(target_arch = "wasm32")]
    fn main() {
        stdweb::initialize();
        println!("hello");
    }
}

#[cfg(target_arch = "wasm32")]
pub use webgl::main;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    unimplemented!("not implemented for native applications")
}
