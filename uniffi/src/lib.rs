use gribberish::message::Message;
pub use gribberish::message::Message;

pub fn parse_grib_message(data: Vec<u8>, offset: u8) -> Option<Message> {
    None
}

pub fn parse_grib_messages(data: Vec<u8>) -> Option<Message> {
    vec![]
}


uniffi_macros::include_scaffolding!("grib_message");