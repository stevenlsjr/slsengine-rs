extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;

use log::{debug, Level};
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;
use wee_alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn wasm_init() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(Level::Debug).unwrap();
}

#[wasm_bindgen]
pub fn run_renderer(elt: HtmlCanvasElement) {
    debug!("hello! {:?}", elt);
}

#[wasm_bindgen_test]
fn test_hello() {
    assert_eq!(1, 1);
}
