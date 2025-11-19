/// GRIB1 Bitmap Section (Section 3)
///
/// The BMS indicates which grid points contain valid data.
/// If a bit is 1, the corresponding grid point has data.
/// If a bit is 0, the grid point is missing/undefined.

use crate::utils::convert::read_u24_from_bytes;

#[derive(Debug, Clone)]
pub struct Grib1BitmapSection {
    data: Vec<u8>,
}

impl Grib1BitmapSection {
    pub fn from_data(data: &[u8]) -> Result<Self, String> {
        if data.len() < 6 {
            return Err("BMS too short".to_string());
        }

        Ok(Grib1BitmapSection {
            data: data.to_vec(),
        })
    }

    /// Get section length
    pub fn length(&self) -> usize {
        read_u24_from_bytes(&self.data, 0).unwrap_or(0) as usize
    }

    /// Get number of unused bits at end of section
    pub fn unused_bits(&self) -> u8 {
        self.data[3]
    }

    /// Get bitmap indicator (0 or 254 = bitmap follows, 255 = predefined)
    pub fn bitmap_indicator(&self) -> u16 {
        ((self.data[4] as u16) << 8) | (self.data[5] as u16)
    }

    /// Check if point at given index is valid (has data)
    pub fn is_valid(&self, index: usize) -> bool {
        if self.bitmap_indicator() == 255 {
            // Predefined bitmap or no bitmap
            return true;
        }

        let byte_offset = 6 + (index / 8);
        if byte_offset >= self.data.len() {
            return false;
        }

        let bit_offset = 7 - (index % 8);
        (self.data[byte_offset] & (1 << bit_offset)) != 0
    }

    /// Generate a vector of valid indices
    pub fn valid_indices(&self, num_points: usize) -> Vec<usize> {
        (0..num_points)
            .filter(|&i| self.is_valid(i))
            .collect()
    }

    /// Count number of valid points
    pub fn count_valid(&self, num_points: usize) -> usize {
        (0..num_points).filter(|&i| self.is_valid(i)).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitmap_all_valid() {
        let mut data = vec![0u8; 10];
        data[0..3].copy_from_slice(&[0x00, 0x00, 0x0A]); // Length = 10
        data[3] = 0; // No unused bits
        data[4..6].copy_from_slice(&[0x00, 0x00]); // Bitmap follows
        data[6..10].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]); // All bits set

        let bms = Grib1BitmapSection::from_data(&data).unwrap();

        // First 32 points should all be valid
        for i in 0..32 {
            assert!(bms.is_valid(i));
        }
    }

    #[test]
    fn test_bitmap_some_invalid() {
        let mut data = vec![0u8; 7];
        data[0..3].copy_from_slice(&[0x00, 0x00, 0x07]); // Length = 7
        data[3] = 0;
        data[4..6].copy_from_slice(&[0x00, 0x00]);
        data[6] = 0b10101010; // Alternating pattern

        let bms = Grib1BitmapSection::from_data(&data).unwrap();

        assert!(bms.is_valid(0));  // 1
        assert!(!bms.is_valid(1)); // 0
        assert!(bms.is_valid(2));  // 1
        assert!(!bms.is_valid(3)); // 0
    }
}
