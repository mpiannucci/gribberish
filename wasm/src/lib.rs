mod utils;

use wasm_bindgen::prelude::*;
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