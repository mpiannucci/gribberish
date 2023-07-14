use std::convert::TryFrom;
use crate::{message::Message, message_metadata::MessageMetadata};

pub struct DataMessage {
    pub metadata: MessageMetadata,
    pub data: Vec<f64>
}

impl <'a> TryFrom<&Message<'a>> for DataMessage {
    type Error = String;

    fn try_from(message: &Message) -> Result<Self, Self::Error> { 
        let metadata = MessageMetadata::try_from(message)?;
        Ok(DataMessage {
            metadata, 
            data: message.data()?,
        })
    }
}

impl <'a> TryFrom<(&Message<'a>, &MessageMetadata)> for DataMessage {
    type Error = String;

    fn try_from(message: (&Message<'a>, &MessageMetadata)) -> Result<Self, Self::Error> { 
        Ok(DataMessage {
            metadata: message.1.clone(),
            data: message.0.data()?,
        })
    }
}
