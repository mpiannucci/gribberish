use super::section::{Section, section_length};
use crate::utils::bit_array_from_bytes;

pub struct DataSection<'a> {
    data: &'a[u8],
    data_template_number: u8,
}

impl Section for DataSection<'_> {
    fn data(&self) -> &[u8] {
        self.data
    }
}

impl<'a> DataSection<'a> {
    pub fn from_data(data: &[u8], offset: usize, data_template_number: u8) -> DataSection {
        let len = section_length(data, offset);
        DataSection {
            data: &data[offset .. offset+len],
            data_template_number,
        }
    }

    pub fn raw_data_array(&self) -> &[u8] {
        &self.data[5..]
    }

    pub fn raw_bit_data(&self) -> Vec<u8> {
        bit_array_from_bytes(self.raw_data_array())
    }
}