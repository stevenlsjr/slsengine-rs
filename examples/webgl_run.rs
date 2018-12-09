#[macro_use]
extern crate stdweb;

#[cfg(target_arch = "wasm32")]
fn main() {
    println!("hello");
    js! {
        alert("Hello from rust");
    }
}
