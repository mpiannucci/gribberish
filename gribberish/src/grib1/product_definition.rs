/// GRIB1 Product Definition Section (Section 1)
///
/// The PDS contains metadata about the data field:
/// - Parameter identification
/// - Level information
/// - Time information
/// - Center and process IDs

use chrono::{DateTime, Duration, TimeZone, Utc};
use crate::utils::convert::{read_u16_from_bytes, read_u24_from_bytes};

#[derive(Debug, Clone)]
pub struct Grib1ProductDefinitionSection {
    data: Vec<u8>,
}

impl Grib1ProductDefinitionSection {
    pub fn from_data(data: &[u8]) -> Result<Self, String> {
        if data.len() < 28 {
            return Err("PDS too short".to_string());
        }

        Ok(Grib1ProductDefinitionSection {
            data: data.to_vec(),
        })
    }

    /// Get section length in bytes
    pub fn length(&self) -> usize {
        read_u24_from_bytes(&self.data, 0).unwrap_or(0) as usize
    }

    /// Get parameter table version number
    pub fn parameter_table_version(&self) -> u8 {
        self.data[3]
    }

    /// Get originating/generating center
    pub fn center_id(&self) -> u8 {
        self.data[4]
    }

    /// Get generating process ID
    pub fn generating_process_id(&self) -> u8 {
        self.data[5]
    }

    /// Get grid identification
    pub fn grid_id(&self) -> u8 {
        self.data[6]
    }

    /// Check if Grid Description Section (GDS) is present
    pub fn has_gds(&self) -> bool {
        (self.data[7] & 0x80) != 0
    }

    /// Check if Bitmap Section (BMS) is present
    pub fn has_bms(&self) -> bool {
        (self.data[7] & 0x40) != 0
    }

    /// Get parameter indicator (variable type)
    pub fn parameter(&self) -> u8 {
        self.data[8]
    }

    /// Get type of level/layer
    pub fn level_type(&self) -> u8 {
        self.data[9]
    }

    /// Get level or layer value
    pub fn level_value(&self) -> u16 {
        read_u16_from_bytes(&self.data, 10).unwrap_or(0)
    }

    /// Get first level value (for layer types)
    pub fn level_1(&self) -> u8 {
        self.data[10]
    }

    /// Get second level value (for layer types)
    pub fn level_2(&self) -> u8 {
        self.data[11]
    }

    /// Get reference time (year of century)
    pub fn year_of_century(&self) -> u8 {
        self.data[12]
    }

    /// Get month
    pub fn month(&self) -> u8 {
        self.data[13]
    }

    /// Get day
    pub fn day(&self) -> u8 {
        self.data[14]
    }

    /// Get hour
    pub fn hour(&self) -> u8 {
        self.data[15]
    }

    /// Get minute
    pub fn minute(&self) -> u8 {
        self.data[16]
    }

    /// Get reference date/time as DateTime<Utc>
    pub fn reference_datetime(&self) -> Result<DateTime<Utc>, String> {
        let year_of_century = self.year_of_century() as i32;
        let month = self.month() as u32;
        let day = self.day() as u32;
        let hour = self.hour() as u32;
        let minute = self.minute() as u32;

        // Determine full year (assume 1900-2099 range)
        let year = if year_of_century >= 0 && year_of_century <= 99 {
            // GRIB1 uses 2-digit years: 00-99
            // Convention: 00-49 = 2000-2049, 50-99 = 1950-1999
            if year_of_century <= 49 {
                2000 + year_of_century
            } else {
                1900 + year_of_century
            }
        } else {
            return Err(format!("Invalid year of century: {}", year_of_century));
        };

        Utc.with_ymd_and_hms(year, month, day, hour, minute, 0)
            .single()
            .ok_or_else(|| format!("Invalid date/time: {}-{:02}-{:02} {:02}:{:02}", year, month, day, hour, minute))
    }

    /// Get forecast time unit indicator
    pub fn time_unit(&self) -> u8 {
        self.data[17]
    }

    /// Get P1 - Period of time (or first part)
    pub fn p1(&self) -> u8 {
        self.data[18]
    }

    /// Get P2 - Period of time (or second part)
    pub fn p2(&self) -> u8 {
        self.data[19]
    }

    /// Get time range indicator
    pub fn time_range_indicator(&self) -> u8 {
        self.data[20]
    }

    /// Get forecast datetime based on reference time and forecast period
    pub fn forecast_datetime(&self) -> Result<DateTime<Utc>, String> {
        let reference = self.reference_datetime()?;
        let duration = self.forecast_duration()?;
        Ok(reference + duration)
    }

    /// Calculate forecast duration from P1, P2, and time unit
    pub fn forecast_duration(&self) -> Result<Duration, String> {
        let time_unit = self.time_unit();
        let p1 = self.p1() as i64;
        let time_range = self.time_range_indicator();

        // Convert time unit to Duration
        let unit_duration = match time_unit {
            0 => Duration::minutes(1),
            1 => Duration::hours(1),
            2 => Duration::days(1),
            3 => Duration::days(30),  // Month (approximate)
            4 => Duration::days(365), // Year (approximate)
            10 => Duration::hours(3),
            11 => Duration::hours(6),
            12 => Duration::hours(12),
            13 => Duration::seconds(1),
            _ => return Err(format!("Unsupported time unit: {}", time_unit)),
        };

        // For now, simple handling: use P1 for instantaneous forecasts
        // Time range indicator 0 = forecast product valid at reference + P1
        match time_range {
            0 | 1 => Ok(unit_duration * p1 as i32),
            _ => Ok(unit_duration * p1 as i32), // Simplified for now
        }
    }

    /// Get sub-center ID (if present in extended PDS)
    pub fn sub_center_id(&self) -> Option<u8> {
        if self.length() > 28 {
            Some(self.data[25])
        } else {
            None
        }
    }

    /// Get decimal scale factor
    pub fn decimal_scale_factor(&self) -> i16 {
        if self.length() >= 28 {
            read_u16_from_bytes(&self.data, 26)
                .map(|v| v as i16)
                .unwrap_or(0)
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pds_parsing() {
        // Create a minimal PDS (28 bytes)
        let mut data = vec![0u8; 28];
        data[0..3].copy_from_slice(&[0x00, 0x00, 0x1c]); // Length = 28
        data[3] = 3;    // Table version
        data[4] = 98;   // Center (ECMWF)
        data[8] = 11;   // Parameter (temperature)
        data[9] = 100;  // Level type (isobaric)
        data[10] = 2;   // Level high byte
        data[11] = 0x32; // Level low byte (500 hPa)
        data[12] = 23;  // Year 2023
        data[13] = 11;  // November
        data[14] = 4;   // 4th
        data[15] = 12;  // 12:00
        data[16] = 0;   // :00
        data[17] = 1;   // Hours
        data[18] = 6;   // P1 = 6 hours

        let pds = Grib1ProductDefinitionSection::from_data(&data).unwrap();

        assert_eq!(pds.center_id(), 98);
        assert_eq!(pds.parameter(), 11);
        assert_eq!(pds.level_type(), 100);
        assert_eq!(pds.level_value(), 0x0232);
    }
}
