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
}

fn section_length(data: &[u8], offset: usize) -> usize {
    read_u32_from_bytes(data, offset).unwrap_or(0) as usize
}

fn section_number(data: &[u8], offset: usize) -> u8 {
    data[offset + 4]
}

pub struct Section<'a> {
	data: &'a[u8],
	section: SectionType<'a>
}

impl <'a> Section <'a> {

	pub fn from_data(data: &'a[u8], offset: usize) -> Section<'a> {
        let section_len = section_length(data, offset);
        let section_num = section_number(data, offset);
        let section_data = &data[offset..offset+section_len];
        
        let section = match(section_num) { 
            0 => SectionType::Indicator(IndicatorSection::from_data(section_data)),
            1 => SectionType::Identification(IdentificationSection::from_data(section_data)),
            3 => SectionType::GridDefinition(GridDefinitionSection::from_data(section_data)),
            4 => SectionType::ProductDefinition(ProductDefinitionSection::from_data(section_data)),
            5 => SectionType::DataRepresentation(DataRepresentationSection::from_data(section_data)),
            6 => SectionType::Bitmap(BitmapSection::from_data(section_data)),
            7 => SectionType::Data(DataSection::from_data(section_data)),
            8 => SectionType::End(EndSection::from_data(section_data)),
            _ => SectionType::LocalUse(LocalUseSection::from_data(section_data)),
        };

        Section {
            data: section_data,
            section,
        }
	}

    pub fn length(&self) -> usize {
        read_u32_from_bytes(self.data, 0).unwrap_or(0) as usize
    }

    pub fn number(&self) -> u8 {
        self.data[4]
    }
}
