/// GRIB1 Message
///
/// Represents a complete GRIB1 message with all sections.

use super::{
    binary_data::Grib1BinaryDataSection,
    bitmap::Grib1BitmapSection,
    grid_description::Grib1Grid,
    indicator::Grib1IndicatorSection,
    parameters::{get_level_type_info, get_parameter},
    product_definition::Grib1ProductDefinitionSection,
};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Grib1Message {
    pds: Grib1ProductDefinitionSection,
    grid: Option<Grib1Grid>,
    bitmap: Option<Grib1BitmapSection>,
    bds: Grib1BinaryDataSection,
    _data_offset: usize, // Offset in original file (reserved for future use)
    message_length: usize, // Store message length directly
}

impl Grib1Message {
    /// Parse a GRIB1 message from data
    pub fn from_data(data: &[u8], offset: usize) -> Result<Self, String> {
        if data.len() < offset + 8 {
            return Err("Data too short for GRIB1 message".to_string());
        }

        // Section 0: Indicator
        let indicator = Grib1IndicatorSection::from_data(&data[offset..])?;
        let total_length = indicator.total_length();

        if data.len() < offset + total_length {
            return Err("Incomplete GRIB1 message".to_string());
        }

        let msg_data = &data[offset..offset + total_length];
        let mut pos = 8; // After indicator section

        // Section 1: Product Definition Section (PDS)
        if pos + 3 > msg_data.len() {
            return Err("Message too short for PDS".to_string());
        }

        let pds_length = read_24bit(&msg_data[pos..pos + 3]);
        let pds = Grib1ProductDefinitionSection::from_data(&msg_data[pos..pos + pds_length])?;
        pos += pds_length;

        // Section 2: Grid Description Section (GDS) - optional
        let grid = if pds.has_gds() {
            if pos + 3 > msg_data.len() {
                return Err("Message too short for GDS".to_string());
            }

            let gds_length = read_24bit(&msg_data[pos..pos + 3]);
            let grid = Grib1Grid::from_data(&msg_data[pos..pos + gds_length])?;
            pos += gds_length;
            Some(grid)
        } else {
            None
        };

        // Section 3: Bitmap Section (BMS) - optional
        let bitmap = if pds.has_bms() {
            if pos + 3 > msg_data.len() {
                return Err("Message too short for BMS".to_string());
            }

            let bms_length = read_24bit(&msg_data[pos..pos + 3]);
            let bitmap = Grib1BitmapSection::from_data(&msg_data[pos..pos + bms_length])?;
            pos += bms_length;
            Some(bitmap)
        } else {
            None
        };

        // Section 4: Binary Data Section (BDS)
        if pos + 3 > msg_data.len() {
            return Err("Message too short for BDS".to_string());
        }

        let bds_length = read_24bit(&msg_data[pos..pos + 3]);
        if pos + bds_length > msg_data.len() {
            return Err("Incomplete BDS".to_string());
        }

        let bds = Grib1BinaryDataSection::from_data(&msg_data[pos..pos + bds_length])?;

        Ok(Grib1Message {
            pds,
            grid,
            bitmap,
            bds,
            _data_offset: offset,
            message_length: total_length,
        })
    }

    /// Get total message length
    pub fn length(&self) -> usize {
        self.message_length
    }

    /// Get reference date/time
    pub fn reference_datetime(&self) -> Result<DateTime<Utc>, String> {
        self.pds.reference_datetime()
    }

    /// Get forecast date/time
    pub fn forecast_datetime(&self) -> Result<DateTime<Utc>, String> {
        self.pds.forecast_datetime()
    }

    /// Get parameter information
    pub fn parameter(&self) -> Option<(String, String, String)> {
        let center = self.pds.center_id();
        let table_version = self.pds.parameter_table_version();
        let param_num = self.pds.parameter();

        get_parameter(center, table_version, param_num).map(|p| {
            (
                p.abbreviation.to_string(),
                p.name.to_string(),
                p.units.to_string(),
            )
        })
    }

    /// Get level type and value
    pub fn level_info(&self) -> (String, f64, String) {
        let level_type = self.pds.level_type();
        let (type_name, units) = get_level_type_info(level_type);

        let level_value = match level_type {
            100 => {
                // Isobaric - value is in hPa
                self.pds.level_value() as f64
            }
            103 | 105 => {
                // Fixed height above ground - value is in meters
                self.pds.level_value() as f64
            }
            _ => {
                // For layer types, use level_1 and level_2
                if self.pds.level_1() != 0 || self.pds.level_2() != 0 {
                    (self.pds.level_1() as u16 * 256 + self.pds.level_2() as u16) as f64
                } else {
                    0.0
                }
            }
        };

        (type_name.to_string(), level_value, units.to_string())
    }

    /// Get grid dimensions
    pub fn grid_shape(&self) -> (usize, usize) {
        self.grid
            .as_ref()
            .map(|g| g.dimensions())
            .unwrap_or((0, 0))
    }

    /// Get a reference to the grid description
    pub fn grid(&self) -> Option<&Grib1Grid> {
        self.grid.as_ref()
    }

    /// Get latitude values
    pub fn latitudes(&self) -> Vec<f64> {
        self.grid
            .as_ref()
            .map(|g| g.latitudes())
            .unwrap_or_default()
    }

    /// Get longitude values
    pub fn longitudes(&self) -> Vec<f64> {
        self.grid
            .as_ref()
            .map(|g| g.longitudes())
            .unwrap_or_default()
    }

    /// Extract data values
    pub fn data(&self) -> Result<Vec<f64>, String> {
        let (ni, nj) = self.grid_shape();
        let num_points = ni * nj;

        if num_points == 0 {
            return Err("Grid dimensions not available".to_string());
        }

        // Build bitmap if present
        let bitmap_vec: Option<Vec<bool>> = if let Some(ref bms) = self.bitmap {
            Some((0..num_points).map(|i| bms.is_valid(i)).collect())
        } else {
            None
        };

        let mut values = self.bds.unpack_data(num_points, bitmap_vec.as_deref())?;

        // Apply decimal scale factor: Y_final = Y_unpacked / 10^D
        let decimal_scale = self.pds.decimal_scale_factor();
        if decimal_scale != 0 {
            let scale_divisor = 10_f64.powi(decimal_scale as i32);
            for value in values.iter_mut() {
                if !value.is_nan() {
                    *value /= scale_divisor;
                }
            }
        }

        Ok(values)
    }

    /// Get center ID
    pub fn center_id(&self) -> u8 {
        self.pds.center_id()
    }

    /// Get generating process ID
    pub fn generating_process_id(&self) -> u8 {
        self.pds.generating_process_id()
    }

    /// Check if this message is part of the ECMWF file
    pub fn is_ecmwf(&self) -> bool {
        self.center_id() == 98
    }
}

/// Helper to read 24-bit unsigned integer
fn read_24bit(data: &[u8]) -> usize {
    ((data[0] as usize) << 16) | ((data[1] as usize) << 8) | (data[2] as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_24bit() {
        let data = [0x00, 0x01, 0x00];
        assert_eq!(read_24bit(&data), 256);

        let data = [0x01, 0x00, 0x00];
        assert_eq!(read_24bit(&data), 65536);
    }
}
