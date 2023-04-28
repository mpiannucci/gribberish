use super::bitmap::BitmapSection;
use super::data::DataSection;
use super::data_representation::DataRepresentationSection;
use super::end::EndSection;
use super::grib_section::GribSection;
use super::grid_definition::GridDefinitionSection;
use super::identification::IdentificationSection;
use super::indicator::IndicatorSection;
use super::local_use::LocalUseSection;
use super::product_definition::ProductDefinitionSection;
use crate::utils::read_u32_from_bytes;

pub enum Section<'a> {
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

impl <'a> Section<'a> {
    pub fn from_data(data: &[u8], offset: usize) -> Option<Section> {
        let section_len = section_length(data, offset)?;
        let section_num = section_number(data, offset)?;

        let section_data = &data[offset..offset + section_len];

        match section_num {
            0 => Some(Section::Indicator(IndicatorSection::from_data(section_data))),
            1 => Some(Section::Identification(IdentificationSection::from_data(section_data))),
            2 => Some(Section::LocalUse(LocalUseSection::from_data(section_data))),
            3 => Some(Section::GridDefinition(GridDefinitionSection::from_data(section_data ))),
            4 => Some(Section::ProductDefinition(
                ProductDefinitionSection::from_data(section_data),
            )),
            5 => Some(Section::DataRepresentation(
                DataRepresentationSection::from_data(section_data),
            )),
            6 => Some(Section::Bitmap(BitmapSection::from_data(section_data))),
            7 => Some(Section::Data(DataSection::from_data(section_data))),
            8 => Some(Section::End(EndSection::from_data(section_data))),
            _ => None,
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

fn section_length(data: &[u8], offset: usize) -> Option<usize> {
    if data.len() <= offset + 4 { 
        None
    } else if IndicatorSection::is_indicator_section(data, offset) {
        Some(16)
    } else if EndSection::is_end_section(data, offset) {
        Some(4)
    } else {
        Some(read_u32_from_bytes(data, offset).unwrap_or(0) as usize)
    }
}

fn section_number(data: &[u8], offset: usize) -> Option<u8> {
    if data.len() <= offset + 4 {
        None
    } else if IndicatorSection::is_indicator_section(data, offset) {
        Some(0)
    } else if EndSection::is_end_section(data, offset) {
        Some(8)
    } else {
        Some(data[offset + 4])
    }
}

pub struct SectionIterator<'a> {
    pub data: &'a [u8],
    pub offset: usize,
}

impl <'a> Iterator for SectionIterator<'a> {
    type Item = Section<'a>;

    fn next(&mut self) -> std::option::Option<<Self as std::iter::Iterator>::Item> {
        let section_len = section_length(self.data, self.offset)?;
        let section_num = section_number(self.data, self.offset)?;

        if self.offset + section_len > self.data.len() {
            return None;
        }

        let section_data = &self.data[self.offset..self.offset + section_len];
        self.offset += section_len;

        match section_num {
            0 => Some(Section::Indicator(IndicatorSection::from_data(
                section_data,
            ))),
            1 => Some(Section::Identification(IdentificationSection::from_data(
                section_data,
            ))),
            2 => Some(Section::LocalUse(LocalUseSection::from_data(section_data))),
            3 => Some(Section::GridDefinition(GridDefinitionSection::from_data(
                section_data,
            ))),
            4 => Some(Section::ProductDefinition(
                ProductDefinitionSection::from_data(section_data),
            )),
            5 => Some(Section::DataRepresentation(
                DataRepresentationSection::from_data(section_data),
            )),
            6 => Some(Section::Bitmap(BitmapSection::from_data(section_data))),
            7 => Some(Section::Data(DataSection::from_data(section_data))),
            8 => Some(Section::End(EndSection::from_data(section_data))),
            _ => None,
        }
    }
}
