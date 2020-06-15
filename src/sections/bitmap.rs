use std::vec::Vec;
use crate::utils::bit_array_from_bytes;
use crate::utils::read_u32_from_bytes;
use super::grib_section::GribSection;

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

    pub fn map_data(&self, unmapped_data: Vec<f64>) -> Vec<f64> {
        let mut value_count: usize = 0;

        self.bitmap().iter().map(|b| match b {
            1 => {
                let m = unmapped_data[value_count];
                value_count += 1;
                m
            },
            _ => std::f64::NAN
        }).collect()
    }
}

impl <'a> GribSection for BitmapSection<'a> {
    fn len(&self) -> usize {
        read_u32_from_bytes(&self.data[0..4], 0).unwrap_or(0) as usize
    }

    fn number(&self) -> u8 {
        self.data[4]
    }
}