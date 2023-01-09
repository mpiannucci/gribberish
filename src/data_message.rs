use std::convert::TryFrom;
use crate::{message::Message, message_metadata::MessageMetadata};

pub struct DataMessage {
    pub metadata: MessageMetadata,
    pub data: Vec<Vec<f64>>
}

impl DataMessage {
    pub fn flattened_data(&self) -> Vec<f64> {
        self.data.clone().into_iter().flatten().collect()
    }
}

impl <'a> TryFrom<&Message<'a>> for DataMessage {
    type Error = String;

    fn try_from(message: &Message) -> Result<Self, Self::Error> { 
        let metadata = MessageMetadata::try_from(message)?;
        Ok(DataMessage {
            metadata, 
            data: message.data_grid()?,
        })
    }
}
