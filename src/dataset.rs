use std::convert::TryFrom;
use crate::message::Message;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

use crate::message::MessageIterator;

pub fn map_messages<'a>(data: &'a [u8]) -> HashMap<String, (usize, usize)> {
    let message_iter = MessageIterator::from_data (
        data, 
        0,
    );

    message_iter
        .enumerate()
        .map(|(index, m)| {
            match m.variable_abbrev().or(m.parameter_index()) {
                Ok(var) => (var, (index, m.byte_offset())), 
                Err(_) => ("unknown".into(), (index, m.byte_offset()))
            }
        })
        .collect()
}

pub struct GribMessage {
    pub var: String, 
    pub name: String, 
    pub units: String, 
    pub forecast_date: DateTime<Utc>,
    pub reference_date: DateTime<Utc>, 
    pub latitude: Vec<f64>, 
    pub longitude: Vec<f64>,
    pub data: Vec<f64>
}

impl <'a> TryFrom<Message<'a>> for GribMessage {
    type Error = String;

    fn try_from(message: Message) -> std::result::Result<Self, <Self as std::convert::TryFrom<Message>>::Error> { 
        Ok(GribMessage {
            var: message.variable_abbrev()?, 
            name: message.variable_name()?, 
            units: message.unit()?, 
            forecast_date: message.forecast_date()?, 
            reference_date: message.reference_date()?, 
            latitude: message.latitudes()?, 
            longitude: message.longitudes()?, 
            data: message.data()?,
        })
    }
}
