use crate::allocator::*;
use crate::index_array::*;
use js_sys::Object;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct ObjectIndexArray(IndexArray<Object>);
