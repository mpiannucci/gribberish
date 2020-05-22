use crate::utils::bit_array_from_bytes;
use crate::templates::data::DataTemplate;

pub struct DataSection<'a> {
    data: &'a[u8],
}

impl<'a> DataSection<'a> {
    pub fn from_data(data: &[u8]) -> DataSection {
        DataSection {
            data: &data,
        }
    }

    pub fn raw_data_array(&self) -> &[u8] {
        &self.data[5..]
    }

    pub fn raw_bit_data(&self) -> Vec<u8> {
        bit_array_from_bytes(self.raw_data_array())
    }
}