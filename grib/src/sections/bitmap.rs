use std::vec::Vec;
use std::iter::Iterator;
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
        let mut nan_count: usize = 0;

        let bitmask = self.bitmap();
        let mut data = Vec::new();
        data.resize(bitmask.len(), 0.0);

        for (i, mask) in bitmask.iter().enumerate() {
            data[i] = match mask {
                1 => unmapped_data[i - nan_count],
                _ => {
                    nan_count += 1;
                    std::f64::NAN
                }
            };
        }

        data
    }

    pub fn data_index(&self, index: usize) -> Option<usize> {
        // 110101011
        // 012345678
        // 5 - 2 = 3
        let bitmask = self.bitmap();
        if (bitmask.len() <= index) {
            return None
        }else if bitmask[index] == 0 {
            return None
        }

        let mut nan_count: usize = 0;
        for i in (0..index).rev() {
            if bitmask[i] == 0 {
                nan_count += 1;
            }
        }
 
        Some(index - nan_count)
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