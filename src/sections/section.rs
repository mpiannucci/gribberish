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

pub enum SectionType<'a> {
	Indicator(IndicatorSection<'a>),
	Identification(IdentificationSection<'a>),
	LocalUse(LocalUseSection<'a>),
	GridDefinition(GridDefinitionSection<'a>),
	ProductDefinition(ProductDefinitionSection<'a>),
	DataRepresentation(DataRepresentationSection<'a>),
	Bitmap(BitmapSection<'a>),
    Data(DataSection<'a>),
	End(EndSection<'a>),
    Invalid,
}

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

pub struct Section<'a> {
	data: &'a[u8],
	pub section: SectionType<'a>
}

impl <'a> Section <'a> {

	pub fn from_data(data: &'a[u8], offset: usize) -> Result<Section<'a>, &'static str> {
        // TODO: Handle data sizing errors

        let section_len = section_length(data, offset);
        let section_num = section_number(data, offset);
        let section_data = &data[offset..offset+section_len];

        let section = match section_num { 
            0 => SectionType::Indicator(IndicatorSection::from_data(section_data)),
            1 => SectionType::Identification(IdentificationSection::from_data(section_data)),
            2 => SectionType::LocalUse(LocalUseSection::from_data(section_data)),
            3 => SectionType::GridDefinition(GridDefinitionSection::from_data(section_data)),
            4 => SectionType::ProductDefinition(ProductDefinitionSection::from_data(section_data)),
            5 => SectionType::DataRepresentation(DataRepresentationSection::from_data(section_data)),
            6 => SectionType::Bitmap(BitmapSection::from_data(section_data)),
            7 => SectionType::Data(DataSection::from_data(section_data)),
            8 => SectionType::End(EndSection::from_data(section_data)),
            _ => SectionType::Invalid,
        };

        if let SectionType::Invalid = section {
            Err("invalid section number")
        } else {
            Ok(Section {
                data: section_data,
                section,
            })
        }
	}

    pub fn len(&self) -> usize {
        section_length(self.data, 0)
    }

    pub fn number(&self) -> u8 {
        section_number(self.data, 0)
    }
}
