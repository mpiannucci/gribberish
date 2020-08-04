use crate::utils::{read_u16_from_bytes, read_u32_from_bytes};
use crate::templates::data_representation::{DataRepresentationTemplate, SimpleGridPointDataRepresentationTemplate};
use super::grib_section::GribSection;

pub struct DataRepresentationSection<'a> {
    data: &'a[u8],
}

impl<'a> DataRepresentationSection<'a> {
    pub fn from_data(data: &[u8]) -> DataRepresentationSection {
        DataRepresentationSection {
            data: &data,
        }
    }
    
    pub fn data_point_count(&self) -> usize {
        read_u32_from_bytes(self.data, 5).unwrap_or(0) as usize
    }

    pub fn data_representation_template_number(&self) -> u16 {
        read_u16_from_bytes(self.data, 9).unwrap_or(0)
    }

    pub fn data_representation_template(&self) -> Option<Box<dyn DataRepresentationTemplate<f64> + 'a>> {
        let template_number = self.data_representation_template_number();
        match template_number {
            0 => Some(Box::new(SimpleGridPointDataRepresentationTemplate::new(self.data))),
            _ => None,
        }
    }
}

impl <'a> GribSection for DataRepresentationSection<'a> {
    fn len(&self) -> usize {
        read_u32_from_bytes(&self.data[0..4], 0).unwrap_or(0) as usize
    }

    fn number(&self) -> u8 {
        self.data[4]
    }
}