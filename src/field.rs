use std::convert::TryFrom;
use chrono::prelude::*;
use crate::message::Message;
use crate::sections::indicator::Discipline;
use crate::sections::section::Section;
use crate::sections::grid_definition::GridDefinitionSection;
use crate::sections::product_definition::ProductDefinitionSection;
use crate::sections::bitmap::BitmapSection;
use crate::sections::data::DataSection;
use crate::templates::product::ProductTemplate;

pub struct Field {
    pub discipline: Discipline,
    pub reference_date: DateTime<Utc>,
    pub variable: &'static str,
    pub units: &'static str
}

impl <'a> TryFrom<Message<'a>> for Field {
	type Error = &'static str;

	fn try_from(message: Message) -> Result<Self, Self::Error> {
		let discipline = match message.sections.first().unwrap() {
			Section::Indicator(indicator) => Ok(indicator.discipline()),
			_ => Err("Indicator section not found when reading discipline"),
		}?.clone();

		let reference_date = unwrap_or_return!(message.sections.iter().find_map(|s| match s {
			Section::Identification(identification) => Some(identification.reference_date()),
			_ => None,
		}), "Identification section not found when reading reference date");

		let product_definition = unwrap_or_return!(message.sections.iter().find_map(|s| match s {
			Section::ProductDefinition(product_definition) => Some(product_definition),
			_ => None,
		}), "Product definition section not found when reading variable data");

		let product_template = unwrap_or_return!(match product_definition.product_definition_template(discipline.clone() as u8) {
			ProductTemplate::HorizontalAnalysisForecast(template) => Some(template),
			_ => None,
		}, "Only HorizontalAnalysisForecast templates are supported at this time");

		// let product = product_template.product();

		// TODO: Now that we have the product, find a way to unwrap it and get the vals
		let variable = "HMAX";
		let units = "m";

		Ok(Field {
			discipline,
			reference_date,
			variable, 
			units,
		})
	}
}