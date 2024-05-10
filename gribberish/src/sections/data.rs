use crate::utils::read_u32_from_bytes;
use super::grib_section::GribSection;

pub struct DataSection<'a> {
    data: &'a [u8],
}

impl <'a> DataSection<'a> {
    pub fn from_data(data: &'a [u8]) -> Self {
        DataSection {
            data,
        }
    }

    pub fn raw_data_array(&self) -> &[u8] {
        &self.data[5..]
    }
}

impl <'a> GribSection for DataSection<'a> {
    fn len(&self) -> usize {
        read_u32_from_bytes(&self.data[0..4], 0).unwrap_or(0) as usize
    }

    fn number(&self) -> u8 {
        self.data[4]
    }
}