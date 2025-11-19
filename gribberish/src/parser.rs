//! GRIB parsing functions
//!
//! This module provides functions for parsing both GRIB1 and GRIB2 files
//! using pure Rust implementations without external dependencies.

use crate::error::GribberishError;
use crate::grib1::Grib1Message;
use crate::message::Message;
use crate::message_metadata::MessageMetadata;
use std::collections::HashMap;

/// Detect GRIB edition from data at offset
fn detect_edition(data: &[u8], offset: usize) -> Option<u8> {
    if offset + 8 > data.len() {
        return None;
    }

    // Check for GRIB magic
    if &data[offset..offset + 4] != b"GRIB" {
        return None;
    }

    // Edition is at byte 7 (0-indexed)
    Some(data[offset + 7])
}

/// Parse only the metadata of a GRIB message without decompressing data
///
/// This is much faster than `parse_message` because it doesn't decompress
/// the data values. Use this for scanning/indexing files.
///
/// # Arguments
/// * `data` - The complete GRIB file data
/// * `offset` - The byte offset where the message starts
///
/// # Returns
/// A tuple of (metadata, message_length) or an error
pub fn parse_message_metadata_only(
    data: &[u8],
    offset: usize,
) -> Result<(MessageMetadata, usize), GribberishError> {
    // Detect GRIB edition
    let edition = detect_edition(data, offset)
        .ok_or_else(|| GribberishError::MessageError("Not a valid GRIB message".to_string()))?;

    match edition {
        1 => {
            // Parse GRIB1 message (metadata only)
            let message = Grib1Message::from_data(data, offset)
                .map_err(|e| GribberishError::MessageError(format!("GRIB1 parse error: {}", e)))?;

            let metadata = grib1_to_metadata(&message)?;
            let length = message.length();

            Ok((metadata, length))
        }
        2 => {
            // Parse GRIB2 message (metadata only)
            let message = Message::from_data(data, offset)
                .ok_or_else(|| GribberishError::MessageError("Failed to parse GRIB2 message".to_string()))?;

            let metadata = MessageMetadata::try_from(&message)?;
            let length = message.len();

            Ok((metadata, length))
        }
        _ => Err(GribberishError::MessageError(format!(
            "Unsupported GRIB edition: {}",
            edition
        ))),
    }
}

/// Parse a single GRIB message at the given offset
///
/// Automatically detects whether it's GRIB1 or GRIB2 and parses accordingly.
///
/// # Arguments
/// * `data` - The complete GRIB file data
/// * `offset` - The byte offset where the message starts
///
/// # Returns
/// A tuple of (metadata, data_values, message_length) or an error
pub fn parse_message(
    data: &[u8],
    offset: usize,
) -> Result<(MessageMetadata, Vec<f64>, usize), GribberishError> {
    // Detect GRIB edition
    let edition = detect_edition(data, offset)
        .ok_or_else(|| GribberishError::MessageError("Not a valid GRIB message".to_string()))?;

    match edition {
        1 => {
            // Parse GRIB1 message
            let message = Grib1Message::from_data(data, offset)
                .map_err(|e| GribberishError::MessageError(format!("GRIB1 parse error: {}", e)))?;

            let metadata = grib1_to_metadata(&message)?;
            let values = message.data()
                .map_err(|e| GribberishError::MessageError(format!("Failed to extract GRIB1 data: {}", e)))?;
            let length = message.length();

            Ok((metadata, values, length))
        }
        2 => {
            // Parse GRIB2 message
            let message = Message::from_data(data, offset)
                .ok_or_else(|| GribberishError::MessageError("Failed to parse GRIB2 message".to_string()))?;

            let metadata = MessageMetadata::try_from(&message)?;
            let values = message.data()?;
            let length = message.len();

            Ok((metadata, values, length))
        }
        _ => Err(GribberishError::MessageError(format!(
            "Unsupported GRIB edition: {}",
            edition
        ))),
    }
}

/// Scan for all GRIB messages in the data
///
/// Quickly scans through the data to find all GRIB message locations
/// without fully parsing them. Supports both GRIB1 and GRIB2.
///
/// # Arguments
/// * `data` - The complete GRIB file data
///
/// # Returns
/// A vector of (offset, message_length) tuples
pub fn scan_messages(data: &[u8]) -> Vec<(usize, usize)> {
    let mut results = Vec::new();
    let mut offset = 0;

    while offset < data.len() {
        // Look for GRIB magic
        let grib_pos = data[offset..].iter().position(|&b| b == b'G');

        if grib_pos.is_none() {
            break;
        }

        let pos = offset + grib_pos.unwrap();

        // Check if this is actually a GRIB message
        if pos + 8 > data.len() || &data[pos..pos + 4] != b"GRIB" {
            offset = pos + 1;
            continue;
        }

        let edition = data[pos + 7];

        match edition {
            1 => {
                // GRIB1: read 3-byte length at offset 4-6
                if pos + 7 <= data.len() {
                    let length = ((data[pos + 4] as usize) << 16)
                        | ((data[pos + 5] as usize) << 8)
                        | (data[pos + 6] as usize);

                    if pos + length <= data.len() {
                        results.push((pos, length));
                        offset = pos + length;
                        continue;
                    }
                }
                offset = pos + 1;
            }
            2 => {
                // GRIB2: use existing iterator
                if let Some(message) = Message::from_data(data, pos) {
                    let length = message.len();
                    results.push((pos, length));
                    offset = pos + length;
                } else {
                    offset = pos + 1;
                }
            }
            _ => {
                offset = pos + 1;
            }
        }
    }

    results
}

/// Convert a GRIB1 message to MessageMetadata format
fn grib1_to_metadata(message: &Grib1Message) -> Result<MessageMetadata, GribberishError> {
    use crate::templates::product::tables::{FixedSurfaceType, GeneratingProcess, TimeUnit};
    use crate::utils::iter::projection::LatLngProjection;

    // Get parameter info
    let (var_abbrev, var_name, units) = message
        .parameter()
        .unwrap_or(("unknown".to_string(), "Unknown".to_string(), "".to_string()));

    // Get level info
    let (level_type_name, level_value, _level_units) = message.level_info();

    // Map GRIB1 level types to GRIB2 FixedSurfaceType
    let fixed_surface_type = match level_type_name.as_str() {
        "surface" => FixedSurfaceType::GroundOrWater,
        "isobaric" => FixedSurfaceType::IsobaricSurface,
        "fixed_height" => FixedSurfaceType::SpecificAltitudeAboveMeanSeaLevel,
        "fixed_height_above_ground" => FixedSurfaceType::SpecifiedHeightLevelAboveGround,
        "mean_sea_level" => FixedSurfaceType::MeanSeaLevel,
        "sigma" => FixedSurfaceType::SigmaLevel,
        "hybrid" => FixedSurfaceType::HybridLevel,
        "isotherm_zero" => FixedSurfaceType::ZeroDegreeIsotherm,
        "cloud_base" => FixedSurfaceType::CloudBase,
        "cloud_top" => FixedSurfaceType::CloudTop,
        "entire_atmosphere" => FixedSurfaceType::EntireAtmosphere,
        _ => FixedSurfaceType::Missing,
    };

    let first_fixed_surface_value = if level_value > 0.0 {
        Some(level_value)
    } else {
        None
    };

    // Get dates
    let reference_date = message
        .reference_datetime()
        .map_err(|e| GribberishError::MessageError(format!("Invalid reference date: {}", e)))?;

    let forecast_date = message
        .forecast_datetime()
        .map_err(|e| GribberishError::MessageError(format!("Invalid forecast date: {}", e)))?;

    // Get grid info
    let grid_shape = message.grid_shape();
    let lats = message.latitudes();
    let lons = message.longitudes();

    // Create projection
    let is_regular_grid = !lats.is_empty() && !lons.is_empty();
    let projector = if is_regular_grid && grid_shape.0 > 0 && grid_shape.1 > 0 {
        use crate::utils::iter::projection::{PlateCareeProjection, RegularCoordinateIterator};

        // Calculate step sizes
        let lat_step = if lats.len() > 1 {
            lats[1] - lats[0]
        } else {
            0.0
        };

        let lon_step = if lons.len() > 1 {
            lons[1] - lons[0]
        } else {
            0.0
        };

        let mut proj_params = HashMap::new();
        proj_params.insert("lat_0".to_string(), lats[0]);
        proj_params.insert("lon_0".to_string(), lons[0]);

        LatLngProjection::PlateCaree(PlateCareeProjection {
            latitudes: RegularCoordinateIterator::new(lats[0], lat_step, grid_shape.1),
            longitudes: RegularCoordinateIterator::new(lons[0], lon_step, grid_shape.0),
            projection_name: "latlon".to_string(),
            projection_params: proj_params,
        })
    } else {
        // Empty/invalid grid - create a minimal projection
        use crate::utils::iter::projection::{PlateCareeProjection, RegularCoordinateIterator};
        LatLngProjection::PlateCaree(PlateCareeProjection {
            latitudes: RegularCoordinateIterator::new(0.0, 0.0, 0),
            longitudes: RegularCoordinateIterator::new(0.0, 0.0, 0),
            projection_name: "latlon".to_string(),
            projection_params: HashMap::new(),
        })
    };

    // Generate a unique key for this message
    let key = format!(
        "{}_{}_{}_{}",
        var_abbrev,
        level_type_name,
        level_value as i32,
        reference_date.format("%Y%m%d%H")
    );

    Ok(MessageMetadata {
        key,
        byte_offset: 0, // Will be set by caller
        message_size: message.length(),
        var: var_abbrev,
        name: var_name,
        units,
        generating_process: GeneratingProcess::Forecast, // GRIB1 doesn't distinguish as much
        statistical_process: None, // Could parse from time range indicator
        time_unit: TimeUnit::Hour, // Default, could parse from PDS
        time_increment_unit: None,
        time_interval: 0,
        time_increment_interval: None,
        first_fixed_surface_type: fixed_surface_type,
        first_fixed_surface_value,
        second_fixed_surface_type: FixedSurfaceType::Missing,
        second_fixed_surface_value: None,
        discipline: "meteorological".to_string(),
        category: "unknown".to_string(),
        data_compression: "simple_packing".to_string(),
        has_bitmap: false, // Could check if message has bitmap
        reference_date,
        forecast_date,
        forecast_end_date: None,
        proj: "longlat".to_string(),
        crs: "EPSG:4326".to_string(),
        is_regular_grid,
        grid_shape,
        projector,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_empty() {
        let data = b"";
        let messages = scan_messages(data);
        assert_eq!(messages.len(), 0);
    }

    #[test]
    fn test_detect_edition_grib1() {
        let mut data = vec![0u8; 8];
        data[0..4].copy_from_slice(b"GRIB");
        data[7] = 1;
        assert_eq!(detect_edition(&data, 0), Some(1));
    }

    #[test]
    fn test_detect_edition_grib2() {
        let mut data = vec![0u8; 8];
        data[0..4].copy_from_slice(b"GRIB");
        data[7] = 2;
        assert_eq!(detect_edition(&data, 0), Some(2));
    }
}
