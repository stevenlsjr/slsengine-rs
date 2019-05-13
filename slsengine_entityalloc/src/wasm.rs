use crate::allocator::*;
use crate::index_array::*;
use log::{debug, Level};

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct JsIndexArray(IndexArray<JsValue>);

#[wasm_bindgen]
impl JsIndexArray {
    #[wasm_bindgen(constructor)]
    pub fn new() -> JsIndexArray {
        JsIndexArray(IndexArray::new())
    }

    pub fn get(&self, i: &GenerationalIndex) -> JsValue {
        let res = self.0.get(*i).cloned();
        res.clone().unwrap_or(JsValue::UNDEFINED)
    }

    pub fn set(
        &mut self,
        i: &GenerationalIndex,
        val: JsValue,
    ) -> Result<(), JsValue> {
        use js_sys::Error;

        if val.is_undefined() {
            return Err(Error::new("value must be defined").into());
        }
        self.0.insert(*i, val);
        Ok(())
    }

    pub fn remove(&mut self, i: &GenerationalIndex) -> JsValue {
        self.0.remove(*i).unwrap_or(JsValue::UNDEFINED)
    }
}

#[wasm_bindgen(start)]
pub fn wasm_main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(Level::Debug);
}
