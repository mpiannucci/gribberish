use crate::utils::read_u32_from_bytes;
use super::indicator::IndicatorSection;
use super::identification::IdentificationSection;
use super::local_use::LocalUseSection;
use super::grid_definition::GridDefinitionSection;
use super::product_definition::ProductDefinitionSection;
use super::data_representation::DataRepresentationSection;
use super::data::DataSection;
use super::end::EndSection;

pub enum SectionType<'a> {
	Indicator(IndicatorSection<'a>),
	Identification(IdentificationSection<'a>),
	LocalUse(LocalUseSection<'a>),
	GridDefinition(GridDefinitionSection<'a>),
	ProductDefinition(ProductDefinitionSection<'a>),
	DataRepresentation(DataRepresentationSection<'a>),
	Data(DataSection<'a>),
	End(EndSection<'a>),
}

fn section_length(data: &[u8], offset: usize) -> usize {
    read_u32_from_bytes(data, offset).unwrap_or(0) as usize
}

pub struct Section<'a> {
	data: &'a[u8],
	section: SectionType<'a>
}

impl <'a> Section <'a> {

	pub fn from_data(data: &'a[u8], offset: usize) {
		let section_length = section_length(data, offset);
	}

    pub fn length(&self) -> usize {
        read_u32_from_bytes(self.data, 0).unwrap_or(0) as usize
    }

    pub fn number(&self) -> u8 {
        self.data[4]
    }
}