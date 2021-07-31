use std::vec::Vec;
use std::iter::Iterator;
use crate::utils::bit_array_from_bytes;
use crate::utils::read_u32_from_bytes;
use crate::utils::positive_bit_count;
use crate::utils::byte_to_bits;
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
                // TODO: Breaking here for grabbing all data
                // 'index out of bounds: the len is 16243 but the index is 16243', src/sections/bitmap.rs:47:22
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
        if !self.has_bitmap() {
            return Some(index);
        }

        // 110101011 01101101
        // 012345678
        // 5 - 2 = 3
        let bitmask = self.bitmap();
        if bitmask.len() <= index {
            return None
        } else if bitmask[index] == 0 {
            return None
        }

        let mut nan_count: usize = 0;
        for i in (0..index).rev() {
            if bitmask[i] == 0 {
                nan_count += 1;
            }
        }

        Some(index - nan_count)

        // EXPERIMENTAL
        // let raw_bitmap = self.raw_bitmap_data();
        // let mut index_counter: usize = 0;

        // for i in 0..raw_bitmap.len() {
        //     let valid_count = positive_bit_count(&raw_bitmap[i]);
        //     if index > i * 8 {
        //         // println!("{}: {}", index_counter, valid_count);
        //         index_counter += valid_count as usize;
        //         continue;
        //     }

        //     let bits = byte_to_bits(&raw_bitmap[i]);
        //     let stop_index = (i*8) - index;
        //     index_counter += raw_bitmap[0..stop_index].iter().sum::<u8>() as usize;
        //     break;
        // }

        // Some(index_counter.into())
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