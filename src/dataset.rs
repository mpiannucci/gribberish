use std::collections::HashMap;

use crate::message::MessageIterator;

pub fn map_messages<'a>(data: &'a [u8]) -> HashMap<String, usize> {
    let message_iter = MessageIterator::from_data (
        data, 
        0,
    );

    message_iter
        .map(|m| {
            match m.variable_abbrev().or(m.parameter_index()) {
                Ok(var) => (var, m.byte_offset()), 
                Err(_) => ("unknown".into(), m.byte_offset())
            }
        })
        .collect()
}