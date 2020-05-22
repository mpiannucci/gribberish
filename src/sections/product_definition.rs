use crate::utils::read_u16_from_bytes;
use crate::templates::product::ProductTemplate;

pub struct ProductDefinitionSection<'a> {
    data: &'a[u8],
}

impl<'a> ProductDefinitionSection<'a> {
    pub fn from_data(data: &[u8]) -> ProductDefinitionSection {
        ProductDefinitionSection {
            data: &data,
        }
    }

    pub fn coord_values_after_template(&self) -> u16 {
        read_u16_from_bytes(self.data, 5).unwrap_or(0)
    }

    pub fn product_definition_template_number(&self) -> u16 {
        read_u16_from_bytes(self.data, 7).unwrap_or(0)
    }

    pub fn product_definition_template(&self, discipline: u8) -> ProductTemplate<'a> {
        ProductTemplate::from_template_number(self.product_definition_template_number(), &self.data, discipline)
    }
}