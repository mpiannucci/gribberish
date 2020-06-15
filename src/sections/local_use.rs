use crate::utils::read_u32_from_bytes;
use super::grib_section::GribSection;

pub struct LocalUseSection<'a> {
    data: &'a[u8],
}

impl<'a> LocalUseSection<'a> {
    pub fn from_data(data: &[u8]) -> LocalUseSection {
        LocalUseSection {
            data: &data,
        }
    }
}

impl <'a> GribSection for LocalUseSection<'a> {
    fn len(&self) -> usize {
        read_u32_from_bytes(&self.data[0..4], 0).unwrap_or(0) as usize
    }

    fn number(&self) -> u8 {
        self.data[4]
    }
}