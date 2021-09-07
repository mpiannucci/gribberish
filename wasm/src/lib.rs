mod utils;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use js_sys::Array;
use gribberish::message::Message;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub struct GribMessage {
    inner: Message,
}

#[wasm_bindgen]
impl GribMessage {
    pub fn var_name(&self) -> String {
        self.inner.variable_name().unwrap()
    }

    pub fn var_abbrev(&self) -> String {
        self.inner.variable_abbrev().unwrap()
    }

    pub fn units(&self) -> String {
        self.inner.unit().unwrap()
    }    
}

#[wasm_bindgen]
pub fn parse_grib_message(data: &[u8], offset: usize) -> GribMessage {
    let message = Message::parse(data, offset).unwrap();
    GribMessage {
        inner: message,
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Array<GribMessage>")]
    pub type GribMessageArray;
}

#[wasm_bindgen]
pub fn pase_grib_messages(data: &[u8]) -> GribMessageArray {
    let mut messages = Message::parse_all(data);
    let mut contained_messages = Vec::new();
    while messages.len() > 0 {
        let grib_message = GribMessage {
            inner: messages.pop().unwrap()
        };

        contained_messages.push(grib_message);
    }

    contained_messages
        .into_iter()
        .map(JsValue::from)
        .collect::<Array>()
        .unchecked_into::<GribMessageArray>()
}
