/// GRIB variable and coordinate naming conventions
///
/// This module provides functions to generate variable and coordinate names
/// that match cfgrib's output conventions for better compatibility.

use crate::templates::product::tables::{FixedSurfaceType, TypeOfStatisticalProcessing};

/// Generate a cfgrib-style variable name from GRIB parameter info
///
/// This function creates short, readable variable names that follow cfgrib conventions:
/// - Short parameter abbreviations (e.g., "t", "u", "v", "gh")
/// - Level information embedded where relevant (e.g., "t2m" for 2m temperature)
/// - Statistical processing as prefix (e.g., "avg_t", "mnt2m", "mxt2m")
pub fn cfgrib_variable_name(
    var_abbrev: &str,
    surface_type: &FixedSurfaceType,
    surface_value: Option<f64>,
    statistical_process: Option<&TypeOfStatisticalProcessing>,
) -> String {
    // Map GRIB parameter abbreviations to cfgrib short names
    let base_param = map_parameter_to_short_name(var_abbrev);

    // Build the variable name with level info if needed
    let var_with_level = add_level_info(&base_param, surface_type, surface_value);

    // Add statistical processing prefix if present
    add_statistical_prefix(&var_with_level, statistical_process)
}

/// Map GRIB parameter abbreviations to cfgrib short names
fn map_parameter_to_short_name(var_abbrev: &str) -> String {
    match var_abbrev.to_uppercase().as_str() {
        // Temperature
        "TMP" => "t".to_string(),
        "TMAX" => "mxt2m".to_string(),
        "TMIN" => "mnt2m".to_string(),

        // Wind components
        "UGRD" => "u".to_string(),
        "VGRD" => "v".to_string(),
        "WIND" => "ws".to_string(), // wind speed
        "GUST" => "gust".to_string(),

        // Humidity
        "SPFH" => "q".to_string(), // specific humidity
        "RH" => "r".to_string(), // relative humidity
        "DPT" => "d".to_string(), // dew point temperature

        // Geopotential
        "HGT" => "gh".to_string(), // geopotential height
        "GP" => "z".to_string(), // geopotential

        // Pressure
        "PRMSL" => "prmsl".to_string(), // pressure reduced to MSL
        "PRES" => "sp".to_string(), // surface pressure
        "MSLET" => "mslet".to_string(), // MSLP (Eta model)

        // Precipitation
        "APCP" => "tp".to_string(), // total precipitation
        "NCPCP" => "cp".to_string(), // convective precipitation
        "ACPCP" => "cp".to_string(), // convective precipitation
        "PRATE" => "prate".to_string(), // precipitation rate
        "CPOFP" => "cpofp".to_string(), // percent frozen precipitation

        // Radiation
        "DSWRF" => "ssrd".to_string(), // downward shortwave radiation flux
        "USWRF" => "usrd".to_string(), // upward shortwave radiation flux
        "DLWRF" => "strd".to_string(), // downward longwave radiation flux
        "ULWRF" => "utrd".to_string(), // upward longwave radiation flux
        "NSWRF" => "ssr".to_string(), // net shortwave radiation flux
        "NLWRF" => "str".to_string(), // net longwave radiation flux

        // Cloud cover
        "TCDC" => "tcc".to_string(), // total cloud cover
        "LCDC" => "lcc".to_string(), // low cloud cover
        "MCDC" => "mcc".to_string(), // medium cloud cover
        "HCDC" => "hcc".to_string(), // high cloud cover

        // Snow
        "SNOD" => "sde".to_string(), // snow depth
        "SNOWC" => "snowc".to_string(), // snow cover
        "WEASD" => "sd".to_string(), // water equivalent of snow depth

        // Soil
        "SOILW" => "swvl".to_string(), // soil moisture
        "TSOIL" => "stl".to_string(), // soil temperature

        // Other common parameters
        "VIS" => "vis".to_string(), // visibility
        "THICK" => "thick".to_string(), // thickness
        "CAPE" => "cape".to_string(), // CAPE
        "CIN" => "cin".to_string(), // convective inhibition
        "PWAT" => "tcw".to_string(), // precipitable water
        "VVEL" => "w".to_string(), // vertical velocity

        // If not in mapping, return lowercase version
        _ => var_abbrev.to_lowercase(),
    }
}

/// Add level information to variable name where appropriate
fn add_level_info(
    param: &str,
    surface_type: &FixedSurfaceType,
    surface_value: Option<f64>,
) -> String {
    match surface_type {
        FixedSurfaceType::SpecifiedHeightLevelAboveGround => {
            // For height above ground, embed the height in the variable name
            if let Some(height) = surface_value {
                let height_m = height as i32;
                match (param, height_m) {
                    // Common patterns: 2m temperature, 10m wind, etc.
                    // Use t2m/d2m format to ensure valid Python identifiers
                    ("t", 2) => "t2m".to_string(),
                    ("d", 2) => "d2m".to_string(),
                    ("u", 10) => "u10".to_string(),
                    ("v", 10) => "v10".to_string(),
                    ("ws", 10) => "ws10".to_string(),
                    ("gust", 10) => "gust10".to_string(),
                    // For other heights, use generic format
                    _ => format!("{}{}m", param, height_m),
                }
            } else {
                param.to_string()
            }
        }
        FixedSurfaceType::IsobaricSurface => {
            // For isobaric surfaces, variable name stays simple
            // The level will be a coordinate dimension
            param.to_string()
        }
        FixedSurfaceType::MeanSeaLevel => {
            // MSL variables typically already have descriptive names
            param.to_string()
        }
        FixedSurfaceType::GroundOrWater => {
            // Surface variables
            param.to_string()
        }
        _ => param.to_string(),
    }
}

/// Add statistical processing prefix to variable name
fn add_statistical_prefix(
    var_name: &str,
    statistical_process: Option<&TypeOfStatisticalProcessing>,
) -> String {
    if let Some(stat) = statistical_process {
        match stat {
            TypeOfStatisticalProcessing::Average => {
                // For some variables, use avg_ prefix, for special ones use embedded format
                match var_name {
                    "t2m" => "avg_t2m".to_string(),
                    "d2m" => "avg_d2m".to_string(),
                    _ => format!("avg_{}", var_name),
                }
            }
            TypeOfStatisticalProcessing::Minimum => {
                // Use mn prefix or embedded format
                match var_name {
                    "t2m" => "mnt2m".to_string(),
                    _ => format!("mn_{}", var_name),
                }
            }
            TypeOfStatisticalProcessing::Maximum => {
                // Use mx prefix or embedded format
                match var_name {
                    "t2m" => "mxt2m".to_string(),
                    _ => format!("mx_{}", var_name),
                }
            }
            TypeOfStatisticalProcessing::Accumulation => {
                // Accumulation is often implicit in the variable name (e.g., tp)
                var_name.to_string()
            }
            TypeOfStatisticalProcessing::StandardDeviation => {
                format!("std_{}", var_name)
            }
            TypeOfStatisticalProcessing::RootMeanSquare => {
                format!("rms_{}", var_name)
            }
            _ => var_name.to_string(),
        }
    } else {
        var_name.to_string()
    }
}

/// Get cfgrib-style coordinate name for a fixed surface type
pub fn cfgrib_coordinate_name(surface_type: &FixedSurfaceType) -> &'static str {
    match surface_type {
        FixedSurfaceType::GroundOrWater => "surface",
        FixedSurfaceType::CloudBase => "cloudBase",
        FixedSurfaceType::CloudTop => "cloudTop",
        FixedSurfaceType::MaximumWindLevel => "maxWind",
        FixedSurfaceType::Tropopause => "tropopause",
        FixedSurfaceType::SeaBottom => "seaBottom",
        FixedSurfaceType::EntireAtmosphere => "entireAtmosphere",
        FixedSurfaceType::IsothermalLevel => "isothermZero",
        FixedSurfaceType::MeanSeaLevel => "meanSea",
        FixedSurfaceType::SpecificAltitudeAboveMeanSeaLevel => "altitude",
        FixedSurfaceType::SpecifiedHeightLevelAboveGround => "heightAboveGround",
        FixedSurfaceType::SigmaLevel => "sigma",
        FixedSurfaceType::HybridLevel => "hybrid",
        FixedSurfaceType::EtaLevel => "eta",
        FixedSurfaceType::SnowLevel => "snowLevel",
        FixedSurfaceType::SigmaHeightLevel => "sigmaHeight",
        FixedSurfaceType::GeneralizedVerticalHeightCoordinate => "heightAboveGround",
        FixedSurfaceType::DepthBelowSeaLevel => "depthBelowSea",
        FixedSurfaceType::DepthBelowWaterSurface => "depthBelowWater",
        FixedSurfaceType::MixingLayer => "mixingLayer",
        FixedSurfaceType::EntireAtmosphereAsSingleLayer => "atmosphere",
        FixedSurfaceType::EntireOceanAsSingleLayer => "ocean",
        FixedSurfaceType::OrderedSequence => "sequence",
        FixedSurfaceType::Missing => "",
        FixedSurfaceType::ZeroDegreeIsotherm => "isothermZero",
        FixedSurfaceType::AdiabaticCondensationLifted => "adiabaticCondensation",
        FixedSurfaceType::NominalTopOfAtmosphere => "nominalTop",
        FixedSurfaceType::LevelOfFreeConvection => "levelOfFreeConvection",
        FixedSurfaceType::DepthBelowLandSurface => "depthBelowLand",
        FixedSurfaceType::HighestTroposphericFreezingLevel => "heightAboveGround",
        FixedSurfaceType::BoundaryLayerCloudLayer => "cloudLayer",
        FixedSurfaceType::LowCloudLayer => "lowCloudLayer",
        FixedSurfaceType::CloudCeiling => "cloudCeiling",
        FixedSurfaceType::MiddleCloudLayer => "middleCloudLayer",
        FixedSurfaceType::HighCloudLayer => "highCloudLayer",
        FixedSurfaceType::EquilibriumLevel => "equilibriumLevel",
        FixedSurfaceType::IsobaricSurface => "isobaricInhPa",
        FixedSurfaceType::LevelAtSpecifiedPressureDifferenceFromGroundToLevel => "pressureFromGround",
        FixedSurfaceType::PlanetaryBoundaryLayer => "planetaryBoundaryLayer",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_2m() {
        let name = cfgrib_variable_name(
            "TMP",
            &FixedSurfaceType::SpecifiedHeightLevelAboveGround,
            Some(2.0),
            None,
        );
        assert_eq!(name, "t2m");
    }

    #[test]
    fn test_temperature_2m_average() {
        let name = cfgrib_variable_name(
            "TMP",
            &FixedSurfaceType::SpecifiedHeightLevelAboveGround,
            Some(2.0),
            Some(&TypeOfStatisticalProcessing::Average),
        );
        assert_eq!(name, "avg_t2m");
    }

    #[test]
    fn test_temperature_2m_min() {
        let name = cfgrib_variable_name(
            "TMP",
            &FixedSurfaceType::SpecifiedHeightLevelAboveGround,
            Some(2.0),
            Some(&TypeOfStatisticalProcessing::Minimum),
        );
        assert_eq!(name, "mnt2m");
    }

    #[test]
    fn test_wind_10m() {
        let name = cfgrib_variable_name(
            "UGRD",
            &FixedSurfaceType::SpecifiedHeightLevelAboveGround,
            Some(10.0),
            None,
        );
        assert_eq!(name, "u10");
    }

    #[test]
    fn test_geopotential_height() {
        let name = cfgrib_variable_name(
            "HGT",
            &FixedSurfaceType::IsobaricSurface,
            Some(500.0),
            None,
        );
        assert_eq!(name, "gh");
    }

    #[test]
    fn test_coordinate_name() {
        assert_eq!(
            cfgrib_coordinate_name(&FixedSurfaceType::SpecifiedHeightLevelAboveGround),
            "heightAboveGround"
        );
        assert_eq!(
            cfgrib_coordinate_name(&FixedSurfaceType::IsobaricSurface),
            "isobaricInhPa"
        );
    }
}
