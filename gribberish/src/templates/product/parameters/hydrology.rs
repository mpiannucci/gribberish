use gribberish_macros::{DisplayDescription, FromValue, ToParameter};
use gribberish_types::Parameter;

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter)]
pub enum HydrologyBasicProduct {
    #[description = "flash flood guidance"]
    #[abbrev = "FFLDG"]
    #[unit = "kg m-2"]
    FlashFloodGuidance = 0,
    #[description = "flash flood runoff"]
    #[abbrev = "FFLDRO"]
    #[unit = "kg m-2"]
    FlashFloodRunoff = 1,
    #[description = "remotely sensed snow cover"]
    #[abbrev = "RSSC"]
    #[unit = "%"]
    RemotelySensedSnowCover = 2,
    #[description = "elevation of snow covered terrain"]
    #[abbrev = "ESCT"]
    #[unit = "m"]
    ElevationOfSnowCoveredTerrain = 3,
    #[description = "snow water equivalent percent of normal"]
    #[abbrev = "SWEPON"]
    #[unit = "%"]
    SnowWaterEquivalentPercentOfNormal = 4,
    #[description = "baseflow-groundwater runoff"]
    #[abbrev = "BGRUN"]
    #[unit = "kg m-2"]
    BaseflowGroundwaterRunoff = 5,
    #[description = "storm surface runoff"]
    #[abbrev = "SSRUN"]
    #[unit = "kg m-2"]
    StormSurfaceRunoff = 6,
    Missing = 255,
}

pub fn hydrology_parameter(category: u8, parameter: u8) -> Option<Parameter> {
    match category {
        0 => Some(Parameter::from(HydrologyBasicProduct::from(parameter))),
        _ => None,
    }
}

pub fn hydrology_category(category: u8) -> &'static str {
    match category {
        0 => "hydrology basic",
        1 => "hydrology probabilities",
        _ => "other",
    }
}
