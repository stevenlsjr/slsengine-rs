
#[cfg(test)]
mod test;

#[cfg(target_arch="wasm32")]
#[macro_use]
extern crate stdweb;

#[cfg(target_arch="wasm32")]
mod web {
}

#[cfg(target_arch="wasm32")]
fn main() {
    stdweb::initialize();

    js! {
        alert("hello world!!");
    }

    stdweb::event_loop();

}

#[cfg(not(target_arch="wasm32"))]
fn main(){}