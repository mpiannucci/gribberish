use crate::utils::{read_u16_from_bytes, read_u32_from_bytes};
use crate::templates::product::ProductTemplate;
use super::grib_section::GribSection;

pub struct ProductDefinitionSection{
    data: Vec<u8>,
}

impl ProductDefinitionSection {
    pub fn from_data(data: Vec<u8>) -> ProductDefinitionSection {
        ProductDefinitionSection {
            data: data,
        }
    }

    pub fn coord_values_after_template(&self) -> u16 {
        read_u16_from_bytes(self.data.as_slice(), 5).unwrap_or(0)
    }

    pub fn product_definition_template_number(&self) -> u16 {
        read_u16_from_bytes(self.data.as_slice(), 7).unwrap_or(0)
    }

    pub fn product_definition_template(&self, discipline: u8) -> ProductTemplate {
        ProductTemplate::from_template_number(self.product_definition_template_number(), &self.data, discipline)
    }
}

impl GribSection for ProductDefinitionSection {
    fn len(&self) -> usize {
        read_u32_from_bytes(&self.data[0..4], 0).unwrap_or(0) as usize
    }

    fn number(&self) -> u8 {
        self.data[4]
    }
}