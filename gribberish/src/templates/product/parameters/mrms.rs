use gribberish_macros::{DisplayDescription, FromAbbrevStr, FromValue, ToParameter};
use gribberish_types::Parameter;

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter, FromAbbrevStr)]
pub enum MRMSLightningProduct {
    #[description = "CG Average Lightning Density 1-min"]
    #[abbrev = "NLDN_CG_001min_AvgDensity"]
    #[unit = "flashes/km^2/min"]
    AverageFlashDensityOne = 0,
    #[description = "CG Average Lightning Density 5-min"]
    #[abbrev = "NLDN_CG_001min_AvgDensity"]
    #[unit = "flashes/km^2/min"]
    AverageFlashDensityFive = 1,
    #[description = "CG Average Lightning Density 15-min"]
    #[abbrev = "NLDN_CG_001min_AvgDensity"]
    #[unit = "flashes/km^2/min"]
    AverageFlashDensityFifteen = 2,
    #[description = "CG Average Lightning Density 30-min"]
    #[abbrev = "NLDN_CG_001min_AvgDensity"]
    #[unit = "flashes/km^2/min"]
    AverageFlashDensityThirty = 3,
    #[description = "Lightning Probability 0-30 minutes"]
    #[abbrev = "LightningProbabilityNext30minGrid"]
    #[unit = "%"]
    LightningProbabilityThirty = 5,
    #[description = "Lightning Probability 0-60 minutes"]
    #[abbrev = "LightningProbabilityNext60minGrid"]
    #[unit = "%"]
    LightningProbabilitySixty = 6,
    #[description = "Rapid lightning increases and decreases"]
    #[abbrev = "LightningJumpGrid"]
    #[unit = "NonDim"]
    LightningJumpGrid = 7,
    #[description = "Rapid lightning increases and decreases over 5-minutes"]
    #[abbrev = "LightningJumpGrid_Max_005min"]
    #[unit = "NonDim"]
    LightningJumpGridMax = 8,
    Missing = 255,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter, FromAbbrevStr)]
pub enum MRMSConvectionProduct {
    #[description = "Rotation Track 0-2km AGL 30-min"]
    #[abbrev = "RotationTrack30min"]
    #[unit = "0.001/s"]
    RotationTrack30min = 2,
    #[description = "Rotation Track 0-2km AGL 60-min"]
    #[abbrev = "RotationTrack60min"]
    #[unit = "0.001/s"]
    RotationTrack60min = 3,
    #[description = "Rotation Track 0-2km AGL 120-min"]
    #[abbrev = "RotationTrack120min"]
    #[unit = "0.001/s"]
    RotationTrack120min = 4,
    #[description = "Rotation Track 0-2km AGL 240-min"]
    #[abbrev = "RotationTrack240min"]
    #[unit = "0.001/s"]
    RotationTrack240min = 5,
    #[description = "Rotation Track 0-2km AGL 360-min"]
    #[abbrev = "RotationTrack260min"]
    #[unit = "0.001/s"]
    RotationTrack360min = 6,
    #[description = "Rotation Track 0-2km AGL 1440-min"]
    #[abbrev = "RotationTrack1440min"]
    #[unit = "0.001/s"]
    RotationTrack1440min = 7,
    #[description = "Severe Hail Index"]
    #[abbrev = "SHI"]
    #[unit = "index"]
    SHI = 26,
    #[description = "Probability of severe hail"]
    #[abbrev = "POSH"]
    #[unit = "%"]
    POSH = 27,
    #[description = "Maximum Estimated Size of Hail (MESH)"]
    #[abbrev = "MESH"]
    #[unit = "mm"]
    MESH = 28,
    #[description = "ReflectivityAtLowestAltitude"]
    #[abbrev = "ReflectivityAtLowestAltitude"]
    #[unit = "dBZ"]
    ReflectivityAtLowestAltitude = 57,
    #[description = "Non Quality Controlled Reflectivity At Lowest Altitude"]
    #[abbrev = "MergedReflectivityAtLowestAltitude"]
    #[unit = "dBZ"]
    MergedReflectivityAtLowestAltitude = 58,
    Missing = 255,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter, FromAbbrevStr)]
pub enum MRMSPrecipitationProduct {
    #[description = "Surface Precipitation Type (Convective, Stratiform, Tropical, Hail, Snow)"]
    #[abbrev = "PrecipFlag"]
    #[unit = "flag"]
    PrecipFlag = 0,
    #[description = "Radar Precipitation Rate"]
    #[abbrev = "PrecipRate"]
    #[unit = "mm/hr"]
    PrecipRate = 1,
    #[description = "Radar precipitation accumulation 1-hour"]
    #[abbrev = "RadarOnly_QPE_01H"]
    #[unit = "mm"]
    RadarPrecipOne = 2,
    #[description = "Radar precipitation accumulation 3-hour"]
    #[abbrev = "RadarOnly_QPE_03H"]
    #[unit = "mm"]
    RadarPrecipThree = 3,
    #[description = "Radar precipitation accumulation 6-hour"]
    #[abbrev = "RadarOnly_QPE_06H"]
    #[unit = "mm"]
    RadarPrecipSix = 4,
    #[description = "Radar precipitation accumulation 12-hour"]
    #[abbrev = "RadarOnly_QPE_12H"]
    #[unit = "mm"]
    RadarPrecipTwelve = 5,
    #[description = "Radar precipitation accumulation 24-hour"]
    #[abbrev = "RadarOnly_QPE_24H"]
    #[unit = "mm"]
    RadarPrecipTwentyFour = 6,
    #[description = "Radar precipitation accumulation 48-hour"]
    #[abbrev = "RadarOnly_QPE_48H"]
    #[unit = "mm"]
    RadarPrecipFortyEight = 7,
    #[description = "Radar precipitation accumulation 72-hour"]
    #[abbrev = "RadarOnly_QPE_72H"]
    #[unit = "mm"]
    RadarPrecipSeventyTwo = 8,
    #[description = "Multi-sensor accumulation 1-hour (1-hour latency)"]
    #[abbrev = "MultiSensor_QPE_01H_Pass1"]
    #[unit = "mm"]
    MultiSensorPrecipOnePass1 = 30,
    #[description = "Multi-sensor accumulation 3-hour (1-hour latency)"]
    #[abbrev = "MultiSensor_QPE_03H_Pass1"]
    #[unit = "mm"]
    MultiSensorPrecipThreePass1 = 31,
    #[description = "Multi-sensor accumulation 6-hour (1-hour latency)"]
    #[abbrev = "MultiSensor_QPE_06H_Pass1"]
    #[unit = "mm"]
    MultiSensorPrecipSixPass1 = 32,
    #[description = "Multi-sensor accumulation 12-hour (1-hour latency)"]
    #[abbrev = "MultiSensor_QPE_12H_Pass1"]
    #[unit = "mm"]
    MultiSensorPrecipTwelvePass1 = 33,
    #[description = "Multi-sensor accumulation 24-hour (1-hour latency)"]
    #[abbrev = "MultiSensor_QPE_24H_Pass1"]
    #[unit = "mm"]
    MultiSensorPrecipTwentyFourPass1 = 34,
    #[description = "Multi-sensor accumulation 48-hour (1-hour latency)"]
    #[abbrev = "MultiSensor_QPE_48H_Pass1"]
    #[unit = "mm"]
    MultiSensorPrecipFortyEightPass1 = 35,
    #[description = "Multi-sensor accumulation 72-hour (1-hour latency)"]
    #[abbrev = "MultiSensor_QPE_72H_Pass1"]
    #[unit = "mm"]
    MultiSensorPrecipSeventyTwoPass1 = 36,
    #[description = "Multi-sensor accumulation 1-hour (2-hour latency)"]
    #[abbrev = "MultiSensor_QPE_01H_Pass2"]
    #[unit = "mm"]
    MultiSensorPrecipOnePass2 = 37,
    #[description = "Multi-sensor accumulation 3-hour (2-hour latency)"]
    #[abbrev = "MultiSensor_QPE_03H_Pass2"]
    #[unit = "mm"]
    MultiSensorPrecipThreePass2 = 38,
    #[description = "Multi-sensor accumulation 6-hour (2-hour latency)"]
    #[abbrev = "MultiSensor_QPE_06H_Pass2"]
    #[unit = "mm"]
    MultiSensorPrecipSixPass2 = 39,
    #[description = "Multi-sensor accumulation 12-hour (2-hour latency)"]
    #[abbrev = "MultiSensor_QPE_12H_Pass2"]
    #[unit = "mm"]
    MultiSensorPrecipTwelvePass2 = 40,
    #[description = "Multi-sensor accumulation 24-hour (2-hour latency)"]
    #[abbrev = "MultiSensor_QPE_24H_Pass2"]
    #[unit = "mm"]
    MultiSensorPrecipTwentyFourPass2 = 41,
    #[description = "Multi-sensor accumulation 48-hour (2-hour latency)"]
    #[abbrev = "MultiSensor_QPE_48H_Pass2"]
    #[unit = "mm"]
    MultiSensorPrecipFortyEightPass2 = 42,
    #[description = "Multi-sensor accumulation 72-hour (2-hour latency)"]
    #[abbrev = "MultiSensor_QPE_72H_Pass2"]
    #[unit = "mm"]
    MultiSensorPrecipSeventyTwoPass2 = 43,
    #[description = "Radar precipitation accumulation 15-minute"]
    #[abbrev = "RadarOnly_QPE_15M"]
    #[unit = "mm"]
    RadarPrecipFifteen = 45,
    #[description = "Radar precipitation accumulation since 12Z"]
    #[abbrev = "RadarOnly_QPE_Since12Z"]
    #[unit = "mm"]
    RadarPrecipTwelveZ = 46,
    Missing = 255,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter, FromAbbrevStr)]
pub enum MRMSCompositeReflectivityProduct {
    #[description = "Composite Reflectivity Mosaic (optimal method)"]
    #[abbrev = "MergedReflectivityQCComposite"]
    #[unit = "dBZ"]
    MergedReflectivityQCComposite = 0,
    #[description = "Height of Composite Reflectivity Mosaic (optimal method)"]
    #[abbrev = "HeightCompositeReflectivity"]
    #[unit = "m MSL"]
    HeightCompositeReflectivity = 1,
    #[description = "Low-Level Composite Reflectivity Mosaic (0-4km)"]
    #[abbrev = "LowLevelCompositeReflectivity"]
    #[unit = "dBZ"]
    LowLevelCompositeReflectivity = 2,
    #[description = "Height of Low-Level Composite Reflectivity Mosaic (optimal method)"]
    #[abbrev = "LowLevelCompositeReflectivity"]
    #[unit = "m MSL"]
    HeightLowLevelCompositeReflectivity = 3,
    #[description = "Layer Composite Reflectivity Mosaic 0-24kft (low altitude)"]
    #[abbrev = "LayerCompositeReflectivity_Low"]
    #[unit = "dBZ"]
    LayerCompositeReflectivityLow = 4,
    #[description = "Layer Composite Reflectivity Mosaic 24-60kft (highest altitude)"]
    #[abbrev = "LayerCompositeReflectivity_High"]
    #[unit = "dBZ"]
    LayerCompositeReflectivityHigh = 5,
    #[description = "Layer Composite Reflectivity Mosaic 33-60kft (super-high altitude)"]
    #[abbrev = "LayerCompositeReflectivity_Super"]
    #[unit = "dBZ"]
    LayerCompositeReflectivitySuper = 6,
    #[description = "Composite Reflectivity Hourly Maximum"]
    #[abbrev = "CREF_1HR_MAX"]
    #[unit = "dBZ"]
    CREF1HRMAX = 7,
    #[description = "Base Reflectivity Hourly Maximum"]
    #[abbrev = "BREF_1HR_MAX"]
    #[unit = "dBZ"]
    BREF1HRMAX = 10,
    Missing = 255,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter, FromAbbrevStr)]
pub enum MRMSMergedReflectivityProduct {
    #[description = "Base Reflectivity Mosaic (optimal method)"]
    #[abbrev = "MergedBaseReflectivityQC"]
    #[unit = "dBZ"]
    MergedBaseReflectivityQC = 0,
    #[description = "Raw Composite Reflectivity Mosaic (max ref)"]
    #[abbrev = "MergedReflectivityComposite"]
    #[unit = "dBZ"]
    MergedReflectivityComposite = 1,
    #[description = "Composite Reflectivity Mosaic (max ref)"]
    #[abbrev = "MergedReflectivityQComposite"]
    #[unit = "dBZ"]
    MergedReflectivityQComposite = 2,
    #[description = "Raw Base Reflectivity Mosaic (optimal method)"]
    #[abbrev = "MergedBaseReflectivity"]
    #[unit = "dBZ"]
    MergedBaseReflectivity = 3,
    Missing = 255,
}

pub fn multiradar_parameter(category: u8, parameter: u8) -> Option<Parameter> {
    match category {
        2 => Some(Parameter::from(MRMSLightningProduct::from(parameter))),
        3 => Some(Parameter::from(MRMSConvectionProduct::from(parameter))),
        6 => Some(Parameter::from(MRMSPrecipitationProduct::from(parameter))),
        10 => Some(Parameter::from(MRMSCompositeReflectivityProduct::from(
            parameter,
        ))),
        11 => Some(Parameter::from(MRMSMergedReflectivityProduct::from(
            parameter,
        ))),
        _ => None,
    }
}

pub fn multiradar_category(category: u8) -> &'static str {
    match category {
        2 => "lightning",
        3 => "convection",
        6 => "precipitation",
        10 => "composite reflectivity",
        11 => "merged reflectivity",
        _ => "misc",
    }
}
