/// GRIB1 Parameter Tables
///
/// Maps parameter numbers to variable names and units for different centers.
/// Starting with ECMWF (center 98) parameters.
use super::ecmwf_table_128::ECMWF_TABLE_128;

#[derive(Debug, Clone)]
pub struct Grib1Parameter {
    pub number: u8,
    pub abbreviation: &'static str,
    pub name: &'static str,
    pub units: &'static str,
}

/// Get parameter information for a given center, table version, and parameter number
pub fn get_parameter(center_id: u8, table2_version: u8, parameter: u8) -> Option<Grib1Parameter> {
    match center_id {
        98 => get_ecmwf_parameter(table2_version, parameter), // ECMWF
        7 => get_ncep_parameter(parameter),                   // NCEP
        _ => get_wmo_standard_parameter(parameter),           // WMO standard
    }
}

/// ECMWF parameter table dispatch (center 98)
fn get_ecmwf_parameter(table2_version: u8, parameter: u8) -> Option<Grib1Parameter> {
    match table2_version {
        128 => lookup_table_parameter(ECMWF_TABLE_128, parameter)
            .or_else(|| get_ecmwf_legacy_parameter(parameter)),
        228 => get_ecmwf_table_228_parameter(parameter)
            .or_else(|| get_ecmwf_legacy_parameter(parameter))
            .or_else(|| get_wmo_standard_parameter(parameter)),
        _ => {
            get_ecmwf_legacy_parameter(parameter).or_else(|| get_wmo_standard_parameter(parameter))
        }
    }
}

fn lookup_table_parameter(
    table: &'static [(u8, &'static str, &'static str, &'static str)],
    parameter: u8,
) -> Option<Grib1Parameter> {
    table
        .iter()
        .find(|(number, _, _, _)| *number == parameter)
        .map(|(number, abbreviation, name, units)| Grib1Parameter {
            number: *number,
            abbreviation,
            name,
            units,
        })
}

/// Legacy ECMWF mapping kept for non-128 tables and fallback behavior.
fn get_ecmwf_legacy_parameter(parameter: u8) -> Option<Grib1Parameter> {
    let param = match parameter {
        1 => Grib1Parameter {
            number: 1,
            abbreviation: "sp",
            name: "Surface pressure",
            units: "Pa",
        },
        2 => Grib1Parameter {
            number: 2,
            abbreviation: "prmsl",
            name: "Pressure reduced to MSL",
            units: "Pa",
        },
        11 => Grib1Parameter {
            number: 11,
            abbreviation: "t",
            name: "Temperature",
            units: "K",
        },
        29 => Grib1Parameter {
            number: 29,
            abbreviation: "sst",
            name: "Sea surface temperature",
            units: "K",
        },
        31 => Grib1Parameter {
            number: 31,
            abbreviation: "u",
            name: "U component of wind",
            units: "m s-1",
        },
        32 => Grib1Parameter {
            number: 32,
            abbreviation: "v",
            name: "V component of wind",
            units: "m s-1",
        },
        33 => Grib1Parameter {
            number: 33,
            abbreviation: "u",
            name: "U velocity",
            units: "m s-1",
        },
        34 => Grib1Parameter {
            number: 34,
            abbreviation: "v",
            name: "V velocity",
            units: "m s-1",
        },
        39 => Grib1Parameter {
            number: 39,
            abbreviation: "w",
            name: "Vertical velocity",
            units: "Pa s-1",
        },
        51 => Grib1Parameter {
            number: 51,
            abbreviation: "q",
            name: "Specific humidity",
            units: "kg kg-1",
        },
        52 => Grib1Parameter {
            number: 52,
            abbreviation: "r",
            name: "Relative humidity",
            units: "%",
        },
        53 => Grib1Parameter {
            number: 53,
            abbreviation: "q",
            name: "Humidity mixing ratio",
            units: "kg kg-1",
        },
        54 => Grib1Parameter {
            number: 54,
            abbreviation: "pwat",
            name: "Precipitable water",
            units: "kg m-2",
        },
        59 => Grib1Parameter {
            number: 59,
            abbreviation: "prate",
            name: "Precipitation rate",
            units: "kg m-2 s-1",
        },
        61 => Grib1Parameter {
            number: 61,
            abbreviation: "tp",
            name: "Total precipitation",
            units: "m",
        },
        71 => Grib1Parameter {
            number: 71,
            abbreviation: "tcc",
            name: "Total cloud cover",
            units: "%",
        },
        129 => Grib1Parameter {
            number: 129,
            abbreviation: "z",
            name: "Geopotential",
            units: "m2 s-2",
        },
        130 => Grib1Parameter {
            number: 130,
            abbreviation: "t",
            name: "Temperature",
            units: "K",
        },
        131 => Grib1Parameter {
            number: 131,
            abbreviation: "u",
            name: "U component of wind",
            units: "m s-1",
        },
        132 => Grib1Parameter {
            number: 132,
            abbreviation: "v",
            name: "V component of wind",
            units: "m s-1",
        },
        133 => Grib1Parameter {
            number: 133,
            abbreviation: "q",
            name: "Specific humidity",
            units: "kg kg-1",
        },
        134 => Grib1Parameter {
            number: 134,
            abbreviation: "sp",
            name: "Surface pressure",
            units: "Pa",
        },
        135 => Grib1Parameter {
            number: 135,
            abbreviation: "w",
            name: "Vertical velocity",
            units: "Pa s-1",
        },
        141 => Grib1Parameter {
            number: 141,
            abbreviation: "sd",
            name: "Snow depth",
            units: "m of water equivalent",
        },
        142 => Grib1Parameter {
            number: 142,
            abbreviation: "lsp",
            name: "Large-scale precipitation",
            units: "m",
        },
        143 => Grib1Parameter {
            number: 143,
            abbreviation: "cp",
            name: "Convective precipitation",
            units: "m",
        },
        151 => Grib1Parameter {
            number: 151,
            abbreviation: "prmsl",
            name: "Pressure reduced to MSL",
            units: "Pa",
        },
        157 => Grib1Parameter {
            number: 157,
            abbreviation: "r",
            name: "Relative humidity",
            units: "%",
        },
        165 => Grib1Parameter {
            number: 165,
            abbreviation: "u10",
            name: "10 metre U wind component",
            units: "m s-1",
        },
        166 => Grib1Parameter {
            number: 166,
            abbreviation: "v10",
            name: "10 metre V wind component",
            units: "m s-1",
        },
        167 => Grib1Parameter {
            number: 167,
            abbreviation: "t2m",
            name: "2 metre temperature",
            units: "K",
        },
        168 => Grib1Parameter {
            number: 168,
            abbreviation: "d2m",
            name: "2 metre dewpoint temperature",
            units: "K",
        },
        176 => Grib1Parameter {
            number: 176,
            abbreviation: "ssr",
            name: "Surface net short-wave (solar) radiation",
            units: "J m**-2",
        },
        _ => return None,
    };

    Some(param)
}

/// ECMWF parameter table 228 (center 98, table version 228)
fn get_ecmwf_table_228_parameter(parameter: u8) -> Option<Grib1Parameter> {
    let param = match parameter {
        131 => Grib1Parameter {
            number: 131,
            abbreviation: "u10n",
            name: "10 metre u-component of neutral wind",
            units: "m s**-1",
        },
        246 => Grib1Parameter {
            number: 246,
            abbreviation: "100u",
            name: "100 metre U wind component",
            units: "m s**-1",
        },
        247 => Grib1Parameter {
            number: 247,
            abbreviation: "100v",
            name: "100 metre V wind component",
            units: "m s**-1",
        },
        _ => return None,
    };
    Some(param)
}

/// NCEP parameter table (center 7) - subset of common parameters
fn get_ncep_parameter(parameter: u8) -> Option<Grib1Parameter> {
    get_wmo_standard_parameter(parameter)
}

/// WMO standard parameter table (Table 2)
fn get_wmo_standard_parameter(parameter: u8) -> Option<Grib1Parameter> {
    let param = match parameter {
        1 => Grib1Parameter {
            number: 1,
            abbreviation: "pres",
            name: "Pressure",
            units: "Pa",
        },
        2 => Grib1Parameter {
            number: 2,
            abbreviation: "prmsl",
            name: "Pressure reduced to MSL",
            units: "Pa",
        },
        7 => Grib1Parameter {
            number: 7,
            abbreviation: "gh",
            name: "Geopotential height",
            units: "gpm",
        },
        11 => Grib1Parameter {
            number: 11,
            abbreviation: "t",
            name: "Temperature",
            units: "K",
        },
        33 => Grib1Parameter {
            number: 33,
            abbreviation: "u",
            name: "U-component of wind",
            units: "m s-1",
        },
        34 => Grib1Parameter {
            number: 34,
            abbreviation: "v",
            name: "V-component of wind",
            units: "m s-1",
        },
        39 => Grib1Parameter {
            number: 39,
            abbreviation: "w",
            name: "Vertical velocity",
            units: "Pa s-1",
        },
        51 => Grib1Parameter {
            number: 51,
            abbreviation: "q",
            name: "Specific humidity",
            units: "kg kg-1",
        },
        52 => Grib1Parameter {
            number: 52,
            abbreviation: "r",
            name: "Relative humidity",
            units: "%",
        },
        61 => Grib1Parameter {
            number: 61,
            abbreviation: "tp",
            name: "Total precipitation",
            units: "kg m-2",
        },
        _ => return None,
    };

    Some(param)
}

/// Get level type name and units
pub fn get_level_type_info(level_type: u8) -> (&'static str, &'static str) {
    match level_type {
        1 => ("surface", ""),
        2 => ("cloud_base", ""),
        3 => ("cloud_top", ""),
        4 => ("isotherm_zero", "m"),
        100 => ("isobaric", "hPa"),
        102 => ("mean_sea_level", ""),
        103 => ("fixed_height", "m"),
        105 => ("fixed_height_above_ground", "m"),
        106 => ("sigma", "sigma"),
        107 => ("sigma_height", "sigma"),
        108 => ("sigma_pressure", "sigma"),
        109 => ("hybrid", "hybrid"),
        111 => ("depth_below_surface", "m"),
        112 => ("layer_between_depths", "m"),
        113 => ("isentropic", "K"),
        114 => ("layer_between_isentropic", "K"),
        200 => ("entire_atmosphere", ""),
        201 => ("entire_ocean", ""),
        _ => ("unknown", ""),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ecmwf_parameters() {
        let param = get_parameter(98, 128, 130).unwrap();
        assert_eq!(param.abbreviation, "t");
        assert_eq!(param.name, "Temperature");

        let param = get_parameter(98, 128, 131).unwrap();
        assert_eq!(param.abbreviation, "u");

        // Test table 228 parameters
        let param = get_parameter(98, 228, 246).unwrap();
        assert_eq!(param.abbreviation, "100u");
        assert_eq!(param.name, "100 metre U wind component");

        let param = get_parameter(98, 228, 247).unwrap();
        assert_eq!(param.abbreviation, "100v");
    }

    #[test]
    fn test_ecmwf_table_128_soil_parameters() {
        let stl1 = get_parameter(98, 128, 139).unwrap();
        assert_eq!(stl1.abbreviation, "stl1");
        let swvl1 = get_parameter(98, 128, 39).unwrap();
        assert_eq!(swvl1.abbreviation, "swvl1");
    }

    #[test]
    fn test_ecmwf_non_128_legacy_mappings_preserved() {
        let param = get_parameter(98, 129, 165).unwrap();
        assert_eq!(param.abbreviation, "u10");
    }

    #[test]
    fn test_ecmwf_table_128_entry_count() {
        assert_eq!(ECMWF_TABLE_128.len(), 209);
    }

    #[test]
    fn test_level_types() {
        let (name, units) = get_level_type_info(100);
        assert_eq!(name, "isobaric");
        assert_eq!(units, "hPa");

        let (name, _) = get_level_type_info(1);
        assert_eq!(name, "surface");
    }
}
