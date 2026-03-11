/// ECMWF local GRIB2 parameter support (discipline 192)
///
/// ECMWF encodes many local parameters in GRIB2 using discipline=192.
/// The category number corresponds to the GRIB1 table number (e.g., category=128
/// maps to ECMWF GRIB1 table 128), and the parameter number matches the GRIB1
/// parameter number within that table.
///
/// Reference: ecCodes localConcepts definitions at
/// https://github.com/ecmwf/eccodes/tree/develop/definitions/grib2/localConcepts/ecmf/
use crate::grib1::ecmwf_table_128::ECMWF_TABLE_128;
use crate::grib1::ecmwf_table_140::ECMWF_TABLE_140;
use gribberish_types::Parameter;

/// Look up an ECMWF local parameter from discipline=192 GRIB2 messages.
/// The category corresponds to the GRIB1 table number.
pub fn ecmwf_local_parameter(category: u8, parameter: u8) -> Option<Parameter> {
    let table: &[(u8, &str, &str, &str)] = match category {
        128 => ECMWF_TABLE_128,
        140 => ECMWF_TABLE_140,
        _ => return None,
    };

    table
        .iter()
        .find(|(num, _, _, _)| *num == parameter)
        .map(|(_, abbrev, name, unit)| Parameter {
            name: name.to_string(),
            abbrev: abbrev.to_string(),
            unit: unit.to_string(),
        })
}

pub fn ecmwf_local_category(category: u8) -> &'static str {
    match category {
        128 => "ecmwf table 128",
        140 => "ecmwf table 140",
        _ => "ecmwf local",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ecmwf_discipline_192_table_128() {
        // ECMWF GRIB2 discipline=192, category=128 mirrors GRIB1 table 128
        // e.g., param 139 = Soil temperature level 1
        let stl1 = ecmwf_local_parameter(128, 139).unwrap();
        assert_eq!(stl1.abbrev, "stl1");
        assert!(stl1.name.contains("Soil temperature level 1"));

        // param 39 = Volumetric soil water layer 1
        let swvl1 = ecmwf_local_parameter(128, 39).unwrap();
        assert_eq!(swvl1.abbrev, "swvl1");

        // param 34 = Sea surface temperature
        let sst = ecmwf_local_parameter(128, 34).unwrap();
        assert_eq!(sst.abbrev, "sst");
    }

    #[test]
    fn test_ecmwf_discipline_192_table_140() {
        // ECMWF GRIB2 discipline=192, category=140 mirrors GRIB1 table 140 (waves)
        let swh = ecmwf_local_parameter(140, 229).unwrap();
        assert_eq!(swh.abbrev, "swh");
    }

    #[test]
    fn test_ecmwf_discipline_192_dispatch() {
        use crate::templates::product::parameters::parameter;

        // Verify discipline=192 is wired into the top-level dispatch
        let p = parameter(192, 128, 139).unwrap();
        assert_eq!(p.abbrev, "stl1");

        // Unknown category returns None
        assert!(parameter(192, 99, 0).is_none());
    }
}
