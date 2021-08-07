use crate::utils::read_u32_from_bytes;
use super::indicator::IndicatorSection;
use super::identification::IdentificationSection;
use super::local_use::LocalUseSection;
use super::grid_definition::GridDefinitionSection;
use super::product_definition::ProductDefinitionSection;
use super::data_representation::DataRepresentationSection;
use super::bitmap::BitmapSection;
use super::data::DataSection;
use super::end::EndSection;
use super::grib_section::GribSection;

pub enum Section {
	Indicator(IndicatorSection),
	Identification(IdentificationSection),
	LocalUse(LocalUseSection),
	GridDefinition(GridDefinitionSection),
	ProductDefinition(ProductDefinitionSection),
	DataRepresentation(DataRepresentationSection),
	Bitmap(BitmapSection),
    Data(DataSection),
	End(EndSection),
}

impl Section {
    pub fn from_data(data: &[u8], offset: usize) -> Result<Section, &'static str> {
        let section_len = section_length(data, offset);
        let section_num = section_number(data, offset);

        let section_data = data[offset..offset+section_len].to_vec();

        match section_num { 
            0 => Ok(Section::Indicator(IndicatorSection::from_data(section_data))),
            1 => Ok(Section::Identification(IdentificationSection::from_data(section_data))),
            2 => Ok(Section::LocalUse(LocalUseSection::from_data(section_data))),
            3 => Ok(Section::GridDefinition(GridDefinitionSection::from_data(section_data))),
            4 => Ok(Section::ProductDefinition(ProductDefinitionSection::from_data(section_data))),
            5 => Ok(Section::DataRepresentation(DataRepresentationSection::from_data(section_data))),
            6 => Ok(Section::Bitmap(BitmapSection::from_data(section_data))),
            7 => Ok(Section::Data(DataSection::from_data(section_data))),
            8 => Ok(Section::End(EndSection::from_data(section_data))),
            _ => Err("Invalid section number")
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Section::Indicator(indicator) => indicator.len(),
            Section::Identification(identification) => identification.len(),
            Section::LocalUse(local_use) => local_use.len(),
            Section::GridDefinition(grid_definition) => grid_definition.len(),
            Section::ProductDefinition(product_definition) => product_definition.len(),
            Section::DataRepresentation(data_representation) => data_representation.len(),
            Section::Bitmap(bitmap) => bitmap.len(),
            Section::Data(data) => data.len(),
            Section::End(end) => end.len(),
        }
    }

    pub fn number(&self) -> u8 {
        match self {
            Section::Indicator(indicator) => indicator.number(),
            Section::Identification(identification) => identification.number(),
            Section::LocalUse(local_use) => local_use.number(),
            Section::GridDefinition(grid_definition) => grid_definition.number(),
            Section::ProductDefinition(product_definition) => product_definition.number(),
            Section::DataRepresentation(data_representation) => data_representation.number(),
            Section::Bitmap(bitmap) => bitmap.number(),
            Section::Data(data) => data.number(),
            Section::End(end) => end.number(),
        }
    }
}

// TODO: IMPL TRY FROMS FOR INNER TYPES HERE 

fn section_length(data: &[u8], offset: usize) -> usize {
    if IndicatorSection::is_indicator_section(data, offset) {
        16
    } else if EndSection::is_end_section(data, offset) {
        4
    } else {
        read_u32_from_bytes(data, offset).unwrap_or(0) as usize
    }
}

fn section_number(data: &[u8], offset: usize) -> u8 {
    if IndicatorSection::is_indicator_section(data, offset) {
        0
    } else if EndSection::is_end_section(data, offset) {
        8
    } else {
        data[offset + 4]
    }
}