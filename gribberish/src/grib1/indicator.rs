/// GRIB1 Indicator Section (Section 0)
///
/// The indicator section is 8 bytes and contains:
/// - Bytes 0-3: 'GRIB' magic number
/// - Bytes 4-6: Total message length (3 bytes, big-endian)
/// - Byte 7: GRIB edition number (should be 1)

use crate::utils::read_u24_from_bytes;

pub struct Grib1IndicatorSection<'a> {
    data: &'a [u8],
}

impl<'a> Grib1IndicatorSection<'a> {
    pub fn from_data(data: &'a [u8]) -> Result<Self, String> {
        if data.len() < 8 {
            return Err("Indicator section too short".to_string());
        }

        // Check GRIB magic
        if &data[0..4] != b"GRIB" {
            return Err("Not a GRIB file".to_string());
        }

        // Check edition
        let edition = data[7];
        if edition != 1 {
            return Err(format!("Expected GRIB1, found edition {}", edition));
        }

        Ok(Grib1IndicatorSection { data })
    }

    /// Get total message length in bytes
    pub fn total_length(&self) -> usize {
        read_u24_from_bytes(self.data, 4).unwrap_or(0) as usize
    }

    /// Get GRIB edition number (should be 1)
    pub fn edition(&self) -> u8 {
        self.data[7]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indicator_section() {
        // Create a minimal GRIB1 indicator
        let data = b"GRIB\x00\x00\x40\x01";  // GRIB, length=64, edition=1

        let section = Grib1IndicatorSection::from_data(data).unwrap();
        assert_eq!(section.edition(), 1);
        assert_eq!(section.total_length(), 64);
    }
}
