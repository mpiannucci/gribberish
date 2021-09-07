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

#[wasm_bindgen]
pub fn parse_grib_message(data: &[u8], offset: usize) -> GribMessage {
    let message = Message::parse(data, offset).unwrap();
    GribMessage {
        inner: message,
    }
}

#[wasm_bindgen]
pub struct GribMessages {
    messages: Vec<GribMessage>
}

impl GribMessages {
    pub fn count(&self) -> usize {
        self.messages.len()
    }

    pub fn message_at(&self, index: usize) -> &GribMessage {
        &self.messages[index]
    }
}

#[wasm_bindgen]
pub fn pase_grib_messages(data: &[u8]) -> GribMessages {
    let mut messages = Message::parse_all(data);
    let mut contained_messages = Vec::new();
    while messages.len() > 0 {
        let grib_message = GribMessage {
            inner: messages.pop().unwrap()
        };

        contained_messages.push(grib_message);
    }

    GribMessages {
        messages: contained_messages
    }
}
