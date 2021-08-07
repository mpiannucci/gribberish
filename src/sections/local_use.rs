use crate::utils::read_u32_from_bytes;
use super::grib_section::GribSection;

pub struct LocalUseSection {
    data: Vec<u8>,
}

impl LocalUseSection {
    pub fn from_data(data: Vec<u8>) -> LocalUseSection {
        LocalUseSection {
            data: data,
        }
    }
}

impl GribSection for LocalUseSection {
    fn len(&self) -> usize {
        read_u32_from_bytes(&self.data[0..4], 0).unwrap_or(0) as usize
    }

    fn number(&self) -> u8 {
        self.data[4]
    }
}