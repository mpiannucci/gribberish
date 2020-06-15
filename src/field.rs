use std::convert::TryFrom;
use chrono::prelude::*;
use crate::message::Message;
use crate::sections::indicator::Discipline;
use crate::sections::grid_definition::GridDefinitionSection;
use crate::sections::product_definition::ProductDefinitionSection;
use crate::sections::bitmap::BitmapSection;
use crate::sections::data::DataSection;

pub struct Field {
    pub discipline: Discipline,
    pub reference_date: DateTime<Utc>,
    pub variable: &'static str,
    pub units: &'static str
}

impl <'a> TryFrom<Message<'a>> for Field {
	type Error = &'static str;

	fn try_from(message: Message) -> Result<Self, Self::Error> {
		Err("Not Implemented")
	}
}