//! Eccodes backend implementation
//!
//! This backend uses the ECMWF eccodes library for GRIB parsing.

use crate::backends::GribBackend;
use crate::error::GribberishError;
use crate::message_metadata::MessageMetadata;
use crate::templates::product::tables::{FixedSurfaceType, GeneratingProcess, TimeUnit};
use crate::utils::iter::projection::{
    LatLngProjection, PlateCareeProjection, RegularCoordinateIterator,
};
use chrono::{TimeZone, Utc};
use gribberish_eccodes::{GribHandle, GribMessageIterator};
use std::collections::HashMap;

/// Eccodes-based GRIB parsing backend
///
/// This backend uses the ECMWF eccodes library to parse GRIB files.
/// It provides access to all GRIB features supported by eccodes.
pub struct EccodesBackend;

impl EccodesBackend {
    /// Convert eccodes handle to MessageMetadata
    ///
    /// This maps eccodes keys to our internal metadata structure
    fn handle_to_metadata(
        handle: &GribHandle,
        offset: usize,
    ) -> Result<MessageMetadata, GribberishError> {
        // Get basic identification
        let var = handle
            .get_string("shortName")
            .unwrap_or_else(|_| "unknown".to_string());
        let name = handle
            .get_string("name")
            .unwrap_or_else(|_| "Unknown".to_string());
        let units = handle
            .get_string("units")
            .unwrap_or_else(|_| "unknown".to_string());

        // Get message size
        let message_size = handle
            .get_message_size()
            .map_err(|e| GribberishError::MessageError(e.to_string()))?;

        // Get time information
        let year = handle.get_long("year").unwrap_or(0) as i32;
        let month = handle.get_long("month").unwrap_or(1) as u32;
        let day = handle.get_long("day").unwrap_or(1) as u32;
        let hour = handle.get_long("hour").unwrap_or(0) as u32;
        let minute = handle.get_long("minute").unwrap_or(0) as u32;

        let reference_date = Utc
            .with_ymd_and_hms(year, month, day, hour, minute, 0)
            .single()
            .ok_or_else(|| GribberishError::MessageError("Invalid reference date".to_string()))?;

        // Get forecast time
        let forecast_time = handle.get_long("forecastTime").unwrap_or(0);
        let step_units = handle
            .get_string("stepUnits")
            .unwrap_or_else(|_| "h".to_string());

        // Convert forecast time to duration
        let forecast_duration = match step_units.as_str() {
            "h" => chrono::Duration::hours(forecast_time),
            "m" => chrono::Duration::minutes(forecast_time),
            "s" => chrono::Duration::seconds(forecast_time),
            "d" => chrono::Duration::days(forecast_time),
            _ => chrono::Duration::hours(forecast_time),
        };

        let forecast_date = reference_date + forecast_duration;

        // Get grid dimensions
        let ni = handle.get_long("Ni").unwrap_or(0) as usize;
        let nj = handle.get_long("Nj").unwrap_or(0) as usize;

        // Get discipline and category
        let discipline = handle.get_long("discipline").unwrap_or(0);
        let category = handle.get_long("parameterCategory").unwrap_or(0);

        // Get level information
        let level_type = handle.get_long("typeOfFirstFixedSurface").unwrap_or(0);
        let level_value = handle
            .get_double("scaledValueOfFirstFixedSurface")
            .ok()
            .and_then(|v| {
                let scale = handle.get_long("scaleFactorOfFirstFixedSurface").ok()?;
                Some(v / 10f64.powi(scale as i32))
            });

        let second_level_type = handle.get_long("typeOfSecondFixedSurface").unwrap_or(255);
        let second_level_value = handle
            .get_double("scaledValueOfSecondFixedSurface")
            .ok()
            .and_then(|v| {
                let scale = handle.get_long("scaleFactorOfSecondFixedSurface").ok()?;
                Some(v / 10f64.powi(scale as i32))
            });

        // Get projection information
        let grid_type = handle
            .get_string("gridType")
            .unwrap_or_else(|_| "regular_ll".to_string());

        // Create a simple projection (this is a simplified version)
        // In a full implementation, we'd need to properly construct the projector
        // based on the grid type and parameters
        let proj_string = format!("EPSG:4326"); // Simplified
        let crs = format!("+proj=longlat +datum=WGS84");

        // For now, create a basic lat/lng projector
        // This would need to be expanded to handle different grid types
        let lat1 = handle
            .get_double("latitudeOfFirstGridPointInDegrees")
            .unwrap_or(90.0);
        let lon1 = handle
            .get_double("longitudeOfFirstGridPointInDegrees")
            .unwrap_or(0.0);
        let lat2 = handle
            .get_double("latitudeOfLastGridPointInDegrees")
            .unwrap_or(-90.0);
        let lon2 = handle
            .get_double("longitudeOfLastGridPointInDegrees")
            .unwrap_or(360.0);

        let di = handle.get_double("iDirectionIncrementInDegrees").ok();
        let dj = handle.get_double("jDirectionIncrementInDegrees").ok();

        // Create projector based on grid type
        // Calculate steps if not provided
        let lat_step = if let Some(dj) = dj {
            dj
        } else {
            if nj > 1 {
                (lat2 - lat1) / (nj as f64 - 1.0)
            } else {
                0.0
            }
        };
        let lon_step = if let Some(di) = di {
            di
        } else {
            if ni > 1 {
                (lon2 - lon1) / (ni as f64 - 1.0)
            } else {
                0.0
            }
        };

        let projector = LatLngProjection::PlateCaree(PlateCareeProjection {
            latitudes: RegularCoordinateIterator::new(lat1, lat_step, nj),
            longitudes: RegularCoordinateIterator::new(lon1, lon_step, ni),
            projection_name: "latlon".to_string(),
            projection_params: HashMap::new(),
        });

        // Get compression type
        let packing_type = handle
            .get_string("packingType")
            .unwrap_or_else(|_| "unknown".to_string());

        // Check for bitmap
        let has_bitmap = handle
            .get_long("bitmapPresent")
            .map(|v| v != 0)
            .unwrap_or(false);

        // Get ensemble information
        let product_template_num = handle.get_long("productDefinitionTemplateNumber").unwrap_or(0);
        let is_ensemble = matches!(product_template_num, 1 | 2 | 11 | 12);
        let perturbation_number = if is_ensemble {
            handle.get_long("perturbationNumber").ok().map(|v| v as u8)
        } else {
            None
        };
        let ensemble_size = if is_ensemble {
            handle.get_long("numberOfForecastsInEnsemble").ok().map(|v| v as u8)
        } else {
            None
        };

        // Create a key for this message
        let ensemble_suffix = if let Some(pert) = perturbation_number {
            format!(":ens{}", pert)
        } else {
            "".to_string()
        };

        let key = format!(
            "{}:{}:{}{}",
            var,
            forecast_date.format("%Y%m%d%H%M"),
            level_value.map(|v| format!("{:.0}", v)).unwrap_or_default(),
            ensemble_suffix
        );

        Ok(MessageMetadata {
            key,
            byte_offset: offset,
            message_size,
            var,
            name,
            units,
            generating_process: GeneratingProcess::Analysis, // TODO: Map from eccodes typeOfGeneratingProcess
            statistical_process: None,                        // TODO: Extract from eccodes
            time_unit: TimeUnit::Hour,                        // TODO: Map from stepUnits
            time_increment_unit: None,
            time_interval: forecast_time as u32,
            time_increment_interval: None,
            first_fixed_surface_type: FixedSurfaceType::from(level_type as u8),
            first_fixed_surface_value: level_value,
            second_fixed_surface_type: FixedSurfaceType::from(second_level_type as u8),
            second_fixed_surface_value: second_level_value,
            discipline: format!("{}", discipline),
            category: format!("{}", category),
            data_compression: packing_type,
            has_bitmap,
            reference_date,
            forecast_date,
            forecast_end_date: None, // TODO: Calculate for accumulations
            proj: proj_string,
            crs,
            is_regular_grid: grid_type.contains("regular"),
            grid_shape: (ni, nj),
            projector,
            is_ensemble,
            perturbation_number,
            ensemble_size,
        })
    }
}

impl GribBackend for EccodesBackend {
    fn parse_message(
        &self,
        data: &[u8],
        offset: usize,
    ) -> Result<(MessageMetadata, Vec<f64>, usize), GribberishError> {
        // Create a handle from the data at the given offset
        let message_data = &data[offset..];
        let handle = GribHandle::new_from_message(message_data)
            .map_err(|e| GribberishError::MessageError(e.to_string()))?;

        // Get metadata
        let metadata = Self::handle_to_metadata(&handle, offset)?;

        // Get data values
        let values = handle
            .get_data()
            .map_err(|e| GribberishError::DataRepresentationTemplateError(e.to_string()))?;

        // Get message size
        let size = handle
            .get_message_size()
            .map_err(|e| GribberishError::MessageError(e.to_string()))?;

        Ok((metadata, values, size))
    }

    fn scan_messages(&self, data: &[u8]) -> Vec<(usize, usize)> {
        let mut results = Vec::new();
        let iter = GribMessageIterator::new(data);

        for result in iter {
            if let Ok((handle, offset)) = result {
                if let Ok(size) = handle.get_message_size() {
                    results.push((offset, size));
                }
            }
        }

        results
    }

    fn name(&self) -> &str {
        "eccodes"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eccodes_backend_name() {
        let backend = EccodesBackend;
        assert_eq!(backend.name(), "eccodes");
    }
}
