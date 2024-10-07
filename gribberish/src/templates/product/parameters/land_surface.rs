use gribberish_macros::{DisplayDescription, FromAbbrevStr, FromValue, ToParameter};
use gribberish_types::Parameter;

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter, FromAbbrevStr)]
pub enum VegetationProduct {
    #[description = "land cover"]
    #[abbrev = "lAND"]
    #[unit = "proportion"]
    LandCover = 0,
    #[description = "soil temperature"]
    #[abbrev = "TSOIL"]
    #[unit = "K"]
    SoilTemperature = 2,
    Missing = 255,
}

pub fn land_surface_parameter(category: u8, parameter: u8) -> Option<Parameter> {
    match category {
        0 => Some(Parameter::from(VegetationProduct::from(parameter))),
        _ => None,
    }
}

pub fn land_surface_category(category: u8) -> &'static str {
    match category {
        0 => "vegetation/biomass",
        _ => "other",
    }
}
