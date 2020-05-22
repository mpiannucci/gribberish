use std::vec::Vec;
use crate::utils::bit_array_from_bytes;

pub struct BitmapSection<'a> {
    data: &'a[u8],
}

impl<'a> BitmapSection<'a> {
    pub fn from_data(data: &[u8]) -> BitmapSection {
        BitmapSection {
            data: &data,
        }
    }

    pub fn has_bitmap(&self) -> bool {
        self.data[5] == 0
    }

    pub fn raw_bitmap_data(&self) -> &[u8] {
        &self.data[6..]
    }

    pub fn bitmap(&self) -> Vec<u8> {
        bit_array_from_bytes(self.raw_bitmap_data())
    }
}
