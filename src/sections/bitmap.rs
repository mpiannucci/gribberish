use str;
use char;
use std::vec::Vec;
use super::section::{Section, section_length};
use crate::utils::{read_u16_from_bytes, read_u32_from_bytes};

struct BitmapSection<'a> {
    data: &'a[u8],
}

impl Section for BitmapSection<'_> {
    fn data(&self) -> &[u8] {
        self.data
    }
}

impl<'a> BitmapSection<'a> {
    fn from_data(data: &[u8], offset: usize) -> BitmapSection {
        let len = section_length(data, offset);
        BitmapSection {
            data: &data[offset .. offset+len],
        }
    }

    fn has_bitmap(&self) -> bool {
        self.data[5] == 0
    }

    fn raw_bitmap_data(&self) -> &[u8] {
        &self.data[6..]
    }

    fn bitmap(&self) -> Vec<u8> {
        self.raw_bitmap_data()
            .iter()
            .map(|r| format!("{:b}", r))
            .flat_map(|s| s.chars()
                                    .map(|c| c.to_digit(10).unwrap_or(0) as u8)
                                    .collect::<Vec<u8>>())
            .collect::<Vec<u8>>()
    }
}
