use crate::utils::{bit_array_from_bytes, read_u32_from_bytes};
use super::grib_section::GribSection;

pub struct DataSection {
    data: Vec<u8>,
}

impl DataSection {
    pub fn from_data(data: Vec<u8>) -> DataSection {
        DataSection {
            data: data,
        }
    }

    pub fn raw_data_array(&self) -> &[u8] {
        &self.data[5..]
    }

    pub fn raw_bit_data(&self) -> Vec<u8> {
        bit_array_from_bytes(self.raw_data_array())
    }
}

impl GribSection for DataSection {
    fn len(&self) -> usize {
        read_u32_from_bytes(&self.data[0..4], 0).unwrap_or(0) as usize
    }

    fn number(&self) -> u8 {
        self.data[4]
    }
}