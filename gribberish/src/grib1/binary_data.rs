/// GRIB1 Binary Data Section (Section 4)
///
/// The BDS contains the actual packed data values.
/// This implementation handles simple packing (most common).
use crate::utils::convert::{
    read_grib1_sign_magnitude_i16_from_bytes, read_ibm_f32_from_bytes, read_u24_from_bytes,
};

#[derive(Debug, Clone)]
pub struct Grib1BinaryDataSection {
    data: Vec<u8>,
}

impl Grib1BinaryDataSection {
    pub fn from_data(data: &[u8]) -> Result<Self, String> {
        if data.len() < 11 {
            return Err("BDS too short".to_string());
        }

        Ok(Grib1BinaryDataSection {
            data: data.to_vec(),
        })
    }

    /// Get section length
    pub fn length(&self) -> usize {
        read_u24_from_bytes(&self.data, 0).unwrap_or(0) as usize
    }

    /// Get flag byte
    pub fn flags(&self) -> u8 {
        self.data[3]
    }

    /// Check if grid point data are spherical harmonics
    pub fn is_spherical_harmonics(&self) -> bool {
        (self.flags() & 0x80) != 0
    }

    /// Check if data are complex packing
    pub fn is_complex_packing(&self) -> bool {
        (self.flags() & 0x40) != 0
    }

    /// Check if data use integer packing
    pub fn is_integer_packing(&self) -> bool {
        (self.flags() & 0x20) != 0
    }

    /// Check if data contain additional flags
    pub fn has_additional_flags(&self) -> bool {
        (self.flags() & 0x10) != 0
    }

    /// Get binary scale factor (E)
    pub fn binary_scale_factor(&self) -> i16 {
        // GRIB1 uses sign-magnitude encoding for signed scale factors.
        read_grib1_sign_magnitude_i16_from_bytes(&self.data, 4).unwrap_or(0)
    }

    /// Get reference value (R) - stored in IBM floating point format in GRIB1
    pub fn reference_value(&self) -> f32 {
        read_ibm_f32_from_bytes(&self.data, 6).unwrap_or(0.0)
    }

    /// Get number of bits per value
    pub fn num_bits(&self) -> u8 {
        self.data[10]
    }

    /// Unpack data values
    pub fn unpack_data(
        &self,
        num_values: usize,
        bitmap: Option<&[bool]>,
    ) -> Result<Vec<f64>, String> {
        if self.is_spherical_harmonics() || self.is_complex_packing() {
            return Err("Complex/spherical packing not yet supported".to_string());
        }

        let num_bits = self.num_bits() as usize;
        if num_bits == 0 {
            // All values are the same (reference value)
            return Ok(vec![self.reference_value() as f64; num_values]);
        }

        if num_bits > 32 {
            return Err(format!("Invalid number of bits: {}", num_bits));
        }

        let binary_scale = self.binary_scale_factor();
        let reference = self.reference_value() as f64;
        let scale_factor = 2.0f64.powi(binary_scale as i32);

        // Data starts at byte 11
        let data_offset = 11;

        // Determine how many values to unpack based on bitmap
        let num_packed_values = if let Some(bmap) = bitmap {
            bmap.iter().filter(|&&v| v).count()
        } else {
            num_values
        };

        let mut packed_values = Vec::with_capacity(num_packed_values);

        // Unpack bit-packed values
        let mut bit_offset = 0;
        for _ in 0..num_packed_values {
            let value = read_bits(&self.data, data_offset * 8 + bit_offset, num_bits);
            packed_values.push(value);
            bit_offset += num_bits;
        }

        // Apply formula: Y = R + (X * 2^E)
        // where Y = unpacked value, R = reference, X = packed value, E = binary scale
        let unpacked_values: Vec<f64> = packed_values
            .into_iter()
            .map(|x| reference + (x as f64 * scale_factor))
            .collect();

        // If bitmap exists, expand to full grid with missing values
        if let Some(bmap) = bitmap {
            let mut result = Vec::with_capacity(num_values);
            let mut packed_idx = 0;

            for &is_valid in bmap {
                if is_valid {
                    result.push(unpacked_values[packed_idx]);
                    packed_idx += 1;
                } else {
                    result.push(f64::NAN);
                }
            }

            Ok(result)
        } else {
            Ok(unpacked_values)
        }
    }
}

/// Read a specified number of bits from data starting at bit_offset
fn read_bits(data: &[u8], bit_offset: usize, num_bits: usize) -> u32 {
    if num_bits == 0 || num_bits > 32 {
        return 0;
    }

    let byte_offset = bit_offset / 8;
    let bit_in_byte = bit_offset % 8;

    // Read enough bytes to get all the bits we need
    let bytes_needed = (bit_in_byte + num_bits + 7) / 8;

    if byte_offset + bytes_needed > data.len() {
        return 0;
    }

    // Read up to 5 bytes into a u64
    let mut value = 0u64;
    for i in 0..bytes_needed.min(8) {
        value = (value << 8) | (data[byte_offset + i] as u64);
    }

    // Shift to align the bits we want
    let total_bits = bytes_needed * 8;
    let shift = total_bits - bit_in_byte - num_bits;
    value >>= shift;

    // Mask to get only the bits we want
    let mask = (1u64 << num_bits) - 1;
    (value & mask) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_bits() {
        // Test data: 0b11010011 0b10101100
        let data = vec![0xD3, 0xAC];

        // Read first 4 bits: 1101
        assert_eq!(read_bits(&data, 0, 4), 0b1101);

        // Read next 4 bits: 0011
        assert_eq!(read_bits(&data, 4, 4), 0b0011);

        // Read 8 bits starting at bit 4: 0011_1010
        assert_eq!(read_bits(&data, 4, 8), 0b00111010);

        // Read 3 bits at offset 5: 011
        assert_eq!(read_bits(&data, 5, 3), 0b011);
    }

    /// Helper function to convert IEEE float to IBM float bytes for testing
    fn ieee_to_ibm_bytes(value: f32) -> [u8; 4] {
        if value == 0.0 {
            return [0x00, 0x00, 0x00, 0x00];
        }

        let sign = if value < 0.0 { 0x80 } else { 0x00 };
        let abs_value = value.abs() as f64;

        // IBM: (-1)^sign * mantissa * 16^(exponent - 64)
        // Find exponent and mantissa
        let exponent = ((abs_value.log2() / 4.0).ceil() as i32 + 64) as u8;
        let mantissa = (abs_value / 16_f64.powi(exponent as i32 - 64) * (2_f64.powi(24))) as u32;

        [
            sign | (exponent & 0x7F),
            ((mantissa >> 16) & 0xFF) as u8,
            ((mantissa >> 8) & 0xFF) as u8,
            (mantissa & 0xFF) as u8,
        ]
    }

    #[test]
    fn test_bds_parsing() {
        let mut data = vec![0u8; 20];
        data[0..3].copy_from_slice(&[0x00, 0x00, 0x14]); // Length = 20
        data[3] = 0x00; // Flags (simple packing)
        data[4..6].copy_from_slice(&[0x00, 0x00]); // Binary scale = 0
        data[6..10].copy_from_slice(&ieee_to_ibm_bytes(100.0)); // Reference = 100.0 (IBM format)
        data[10] = 8; // 8 bits per value

        let bds = Grib1BinaryDataSection::from_data(&data).unwrap();

        assert_eq!(bds.binary_scale_factor(), 0);
        assert!((bds.reference_value() - 100.0).abs() < 0.01);
        assert_eq!(bds.num_bits(), 8);
        assert!(!bds.is_complex_packing());
    }

    #[test]
    fn test_binary_scale_sign_magnitude_negative() {
        let mut data = vec![0u8; 20];
        data[0..3].copy_from_slice(&[0x00, 0x00, 0x14]); // Length = 20
        data[3] = 0x00; // Flags (simple packing)
        data[4..6].copy_from_slice(&[0x80, 0x05]); // Binary scale = -5 (sign-magnitude)
        data[6..10].copy_from_slice(&ieee_to_ibm_bytes(0.0));
        data[10] = 8;
        let bds = Grib1BinaryDataSection::from_data(&data).unwrap();
        assert_eq!(bds.binary_scale_factor(), -5);
    }

    #[test]
    fn test_simple_unpacking() {
        let mut data = vec![0u8; 14];
        data[0..3].copy_from_slice(&[0x00, 0x00, 0x0E]); // Length
        data[3] = 0x00; // Simple packing
        data[4..6].copy_from_slice(&[0x00, 0x00]); // Binary scale = 0
        data[6..10].copy_from_slice(&ieee_to_ibm_bytes(0.0)); // Reference = 0 (IBM format)
        data[10] = 8; // 8 bits per value
                      // Packed data: 1, 2, 3
        data[11] = 1;
        data[12] = 2;
        data[13] = 3;

        let bds = Grib1BinaryDataSection::from_data(&data).unwrap();
        let unpacked = bds.unpack_data(3, None).unwrap();

        assert_eq!(unpacked.len(), 3);
        assert!((unpacked[0] - 1.0).abs() < 0.01);
        assert!((unpacked[1] - 2.0).abs() < 0.01);
        assert!((unpacked[2] - 3.0).abs() < 0.01);
    }
}
