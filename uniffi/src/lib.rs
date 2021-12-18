use std::sync::Arc;
use gribberish::message::Message;

pub fn parse_grib_message(data: Vec<u8>, offset: u64) -> Option<Arc<GribMessage>>{
    let message = Message::parse(data.as_slice(), offset as usize);
    match message {
        Ok(m) => Some(Arc::new(GribMessage{inner: m})), 
        Err(_) => None,
    }
}

// pub fn parse_grib_messages(data: Vec<u8>) -> Vec<Message> {
//     vec![]
// }

pub struct GribMessage {
    inner: Message,
}

impl GribMessage {
    pub fn section_count(&self) -> u64 {
        self.inner.section_count() as u64
    }
}


uniffi_macros::include_scaffolding!("gribberish");