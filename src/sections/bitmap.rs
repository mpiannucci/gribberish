use std::vec::Vec;
use super::section::{Section, section_length};
use crate::utils::bits_from_bytes;

pub struct BitmapSection<'a> {
    data: &'a[u8],
}

impl Section for BitmapSection<'_> {
    fn data(&self) -> &[u8] {
        self.data
    }
}

impl<'a> BitmapSection<'a> {
    pub fn from_data(data: &[u8], offset: usize) -> BitmapSection {
        let len = section_length(data, offset);
        BitmapSection {
            data: &data[offset .. offset+len],
        }
    }

    pub fn has_bitmap(&self) -> bool {
        self.data[5] == 0
    }

    pub fn raw_bitmap_data(&self) -> &[u8] {
        &self.data[6..]
    }

    pub fn bitmap(&self) -> Vec<u8> {
        bits_from_bytes(self.raw_bitmap_data())
    }
}
