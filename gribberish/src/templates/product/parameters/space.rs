use gribberish_macros::{DisplayDescription, FromValue, ToParameter};
use gribberish_types::Parameter;

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter)]
pub enum SpaceNcepLocal {
    #[description = "simulated brightness temperature for GOES 12, channel 3"]
    #[abbrev = "SBT123"]
    #[unit = "K"]
    SimulatedBrightnessTempGOES12Ch3 = 1,
    #[description = "simulated brightness temperature for GOES 12, channel 4"]
    #[abbrev = "SBT124"]
    #[unit = "K"]
    SimulatedBrightnessTempGOES12Ch4 = 2,
    #[description = "simulated brightness temperature for GOES 11, channel 3"]
    #[abbrev = "SBT113"]
    #[unit = "K"]
    SimulatedBrightnessTempGOES11Ch3 = 7,
    #[description = "simulated brightness temperature for GOES 11, channel 4"]
    #[abbrev = "SBT114"]
    #[unit = "K"]
    SimulatedBrightnessTempGOES11Ch4 = 8,
    Missing = 255,
}

pub fn space_parameter(category: u8, parameter: u8) -> Option<Parameter> {
    match category {
        192 => Some(Parameter::from(SpaceNcepLocal::from(parameter))),
        _ => None,
    }
}

pub fn space_category(category: u8) -> &'static str {
    match category {
        0 => "image format",
        1 => "quantitative",
        192 => "ncep local",
        _ => "other",
    }
}
