use gribberish_macros::{DisplayDescription, FromValue, ToParameter};
use gribberish_types::Parameter;

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter)]
pub enum MRMSLightningProduct {
    #[description = "CG Average Lightning Density 1-min"]
    #[abbrev = "NLDN_CG_001min_AvgDensity"]
    #[unit = "flashes/km^2/min"]
    AverageFlashDensityOne = 0,
    #[description = "CG Average Lightning Density 5-min"]
    #[abbrev = "NLDN_CG_005min_AvgDensity"]
    #[unit = "flashes/km^2/min"]
    AverageFlashDensityFive = 1,
    #[description = "CG Average Lightning Density 15-min"]
    #[abbrev = "NLDN_CG_015min_AvgDensity"]
    #[unit = "flashes/km^2/min"]
    AverageFlashDensityFifteen = 2,
    #[description = "CG Average Lightning Density 30-min"]
    #[abbrev = "NLDN_CG_030min_AvgDensity"]
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
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter)]
pub enum MRMSConvectionProduct {
    #[description = "Azimuth Shear 0-2km AGL"]
    #[abbrev = "MergedAzShear0to2kmAGL"]
    #[unit = "0.001/s"]
    MergedAzShear0to2kmAGL = 0,
    #[description = "Azimuth Shear 3-6km AGL"]
    #[abbrev = "MergedAzShear3to6kmAGL"]
    #[unit = "0.001/s"]
    MergedAzShear3to6kmAGL = 1,
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
    #[abbrev = "RotationTrack360min"]
    #[unit = "0.001/s"]
    RotationTrack360min = 6,
    #[description = "Rotation Track 0-2km AGL 1440-min"]
    #[abbrev = "RotationTrack1440min"]
    #[unit = "0.001/s"]
    RotationTrack1440min = 7,
    #[description = "Rotation Track 3-6km AGL 30-min"]
    #[abbrev = "RotationTrackML30min"]
    #[unit = "0.001/s"]
    RotationTrackML30min = 14,
    #[description = "Rotation Track 3-6km AGL 60-min"]
    #[abbrev = "RotationTrackML60min"]
    #[unit = "0.001/s"]
    RotationTrackML60min = 15,
    #[description = "Rotation Track 3-6km AGL 120-min"]
    #[abbrev = "RotationTrackML120min"]
    #[unit = "0.001/s"]
    RotationTrackML120min = 16,
    #[description = "Rotation Track 3-6km AGL 240-min"]
    #[abbrev = "RotationTrackML240min"]
    #[unit = "0.001/s"]
    RotationTrackML240min = 17,
    #[description = "Rotation Track 3-6km AGL 360-min"]
    #[abbrev = "RotationTrackML360min"]
    #[unit = "0.001/s"]
    RotationTrackML360min = 18,
    #[description = "Rotation Track 3-6km AGL 1440-min"]
    #[abbrev = "RotationTrackML1440min"]
    #[unit = "0.001/s"]
    RotationTrackML1440min = 19,
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
    #[description = "MESH Hail Swath 30-min"]
    #[abbrev = "MESHMax30min"]
    #[unit = "mm"]
    MESHMax30min = 29,
    #[description = "MESH Hail Swath 60-min"]
    #[abbrev = "MESHMax60min"]
    #[unit = "mm"]
    MESHMax60min = 30,
    #[description = "MESH Hail Swath 120-min"]
    #[abbrev = "MESHMax120min"]
    #[unit = "mm"]
    MESHMax120min = 31,
    #[description = "MESH Hail Swath 240-min"]
    #[abbrev = "MESHMax240min"]
    #[unit = "mm"]
    MESHMax240min = 32,
    #[description = "MESH Hail Swath 360-min"]
    #[abbrev = "MESHMax360min"]
    #[unit = "mm"]
    MESHMax360min = 33,
    #[description = "MESH Hail Swath 1440-min"]
    #[abbrev = "MESHMax1440min"]
    #[unit = "mm"]
    MESHMax1440min = 34,
    #[description = "VIL Swath 120-min"]
    #[abbrev = "VIL_Max_120min"]
    #[unit = "kg/m^2"]
    VILMax120min = 37,
    #[description = "VIL Swath 1440-min"]
    #[abbrev = "VIL_Max_1440min"]
    #[unit = "kg/m^2"]
    VILMax1440min = 40,
    #[description = "Vertically Integrated Liquid"]
    #[abbrev = "VIL"]
    #[unit = "kg/m^2"]
    VIL = 41,
    #[description = "Vertically Integrated Liquid Density"]
    #[abbrev = "VIL_Density"]
    #[unit = "g/m^3"]
    VILDensity = 42,
    #[description = "Vertically Integrated Ice"]
    #[abbrev = "VII"]
    #[unit = "kg/m^2"]
    VII = 43,
    #[description = "Echo Top - 18 dBZ"]
    #[abbrev = "EchoTop_18"]
    #[unit = "km MSL"]
    EchoTop18 = 44,
    #[description = "Echo Top - 30 dBZ"]
    #[abbrev = "EchoTop_30"]
    #[unit = "km MSL"]
    EchoTop30 = 45,
    #[description = "Echo Top - 50 dBZ"]
    #[abbrev = "EchoTop_50"]
    #[unit = "km MSL"]
    EchoTop50 = 46,
    #[description = "Echo Top - 60 dBZ"]
    #[abbrev = "EchoTop_60"]
    #[unit = "km MSL"]
    EchoTop60 = 47,
    #[description = "Thickness [50 dBZ top - (-20C)]"]
    #[abbrev = "H50AboveM20C"]
    #[unit = "km"]
    H50AboveM20C = 48,
    #[description = "Thickness [50 dBZ top - 0C]"]
    #[abbrev = "H50Above0C"]
    #[unit = "km"]
    H50Above0C = 49,
    #[description = "Thickness [60 dBZ top - (-20C)]"]
    #[abbrev = "H60AboveM20C"]
    #[unit = "km"]
    H60AboveM20C = 50,
    #[description = "Thickness [60 dBZ top - 0C]"]
    #[abbrev = "H60Above0C"]
    #[unit = "km"]
    H60Above0C = 51,
    #[description = "Isothermal Reflectivity at 0C"]
    #[abbrev = "Reflectivity_0C"]
    #[unit = "dBZ"]
    Reflectivity0C = 52,
    #[description = "Isothermal Reflectivity at -5C"]
    #[abbrev = "Reflectivity_-5C"]
    #[unit = "dBZ"]
    ReflectivityMinus5C = 53,
    #[description = "Isothermal Reflectivity at -10C"]
    #[abbrev = "Reflectivity_-10C"]
    #[unit = "dBZ"]
    ReflectivityMinus10C = 54,
    #[description = "Isothermal Reflectivity at -15C"]
    #[abbrev = "Reflectivity_-15C"]
    #[unit = "dBZ"]
    ReflectivityMinus15C = 55,
    #[description = "Isothermal Reflectivity at -20C"]
    #[abbrev = "Reflectivity_-20C"]
    #[unit = "dBZ"]
    ReflectivityMinus20C = 56,
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
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter)]
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
    #[description = "Method IDs for blended single and dual-pol derived precip rates"]
    #[abbrev = "SyntheticPrecipRateID"]
    #[unit = "flag"]
    SyntheticPrecipRateID = 44,
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
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter)]
pub enum MRMSModelProduct {
    #[description = "Model Surface temperature"]
    #[abbrev = "Model_SurfaceTemp"]
    #[unit = "C"]
    ModelSurfaceTemp = 0,
    #[description = "Model Surface wet bulb temperature"]
    #[abbrev = "Model_WetBulbTemp"]
    #[unit = "C"]
    ModelWetBulbTemp = 1,
    #[description = "Probability of warm rain"]
    #[abbrev = "WarmRainProbability"]
    #[unit = "%"]
    WarmRainProbability = 2,
    #[description = "Model Freezing Level Height"]
    #[abbrev = "Model_0degC_Height"]
    #[unit = "m MSL"]
    Model0degCHeight = 3,
    #[description = "Brightband Top Height"]
    #[abbrev = "BrightBandTopHeight"]
    #[unit = "m AGL"]
    BrightBandTopHeight = 4,
    #[description = "Brightband Bottom Height"]
    #[abbrev = "BrightBandBottomHeight"]
    #[unit = "m AGL"]
    BrightBandBottomHeight = 5,
    Missing = 255,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter)]
pub enum MRMSQualityProduct {
    #[description = "Radar Quality Index"]
    #[abbrev = "RadarQualityIndex"]
    #[unit = "non-dim"]
    RadarQualityIndex = 0,
    #[description = "Gauge Influence Index for 1-hour QPE (1-hour latency)"]
    #[abbrev = "GaugeInflIndex_01H_Pass1"]
    #[unit = "non-dim"]
    GaugeInflIndex01HPass1 = 1,
    #[description = "Gauge Influence Index for 3-hour QPE (1-hour latency)"]
    #[abbrev = "GaugeInflIndex_03H_Pass1"]
    #[unit = "non-dim"]
    GaugeInflIndex03HPass1 = 2,
    #[description = "Gauge Influence Index for 6-hour QPE (1-hour latency)"]
    #[abbrev = "GaugeInflIndex_06H_Pass1"]
    #[unit = "non-dim"]
    GaugeInflIndex06HPass1 = 3,
    #[description = "Gauge Influence Index for 12-hour QPE (1-hour latency)"]
    #[abbrev = "GaugeInflIndex_12H_Pass1"]
    #[unit = "non-dim"]
    GaugeInflIndex12HPass1 = 4,
    #[description = "Gauge Influence Index for 24-hour QPE (1-hour latency)"]
    #[abbrev = "GaugeInflIndex_24H_Pass1"]
    #[unit = "non-dim"]
    GaugeInflIndex24HPass1 = 5,
    #[description = "Gauge Influence Index for 48-hour QPE (1-hour latency)"]
    #[abbrev = "GaugeInflIndex_48H_Pass1"]
    #[unit = "non-dim"]
    GaugeInflIndex48HPass1 = 6,
    #[description = "Gauge Influence Index for 72-hour QPE (1-hour latency)"]
    #[abbrev = "GaugeInflIndex_72H_Pass1"]
    #[unit = "non-dim"]
    GaugeInflIndex72HPass1 = 7,
    #[description = "Seamless Hybrid Scan Reflectivity with VPR correction"]
    #[abbrev = "SeamlessHSR"]
    #[unit = "dBZ"]
    SeamlessHSR = 8,
    #[description = "Height of Seamless Hybrid Scan Reflectivity"]
    #[abbrev = "SeamlessHSRHeight"]
    #[unit = "km AGL"]
    SeamlessHSRHeight = 9,
    #[description = "Radar 1-hour QPE Accumulation Quality"]
    #[abbrev = "RadarAccumulationQualityIndex_01H"]
    #[unit = "non-dim"]
    RadarAccumulationQualityIndex01H = 10,
    #[description = "Radar 3-hour QPE Accumulation Quality"]
    #[abbrev = "RadarAccumulationQualityIndex_03H"]
    #[unit = "non-dim"]
    RadarAccumulationQualityIndex03H = 11,
    #[description = "Radar 6-hour QPE Accumulation Quality"]
    #[abbrev = "RadarAccumulationQualityIndex_06H"]
    #[unit = "non-dim"]
    RadarAccumulationQualityIndex06H = 12,
    #[description = "Radar 12-hour QPE Accumulation Quality"]
    #[abbrev = "RadarAccumulationQualityIndex_12H"]
    #[unit = "non-dim"]
    RadarAccumulationQualityIndex12H = 13,
    #[description = "Radar 24-hour QPE Accumulation Quality"]
    #[abbrev = "RadarAccumulationQualityIndex_24H"]
    #[unit = "non-dim"]
    RadarAccumulationQualityIndex24H = 14,
    #[description = "Radar 48-hour QPE Accumulation Quality"]
    #[abbrev = "RadarAccumulationQualityIndex_48H"]
    #[unit = "non-dim"]
    RadarAccumulationQualityIndex48H = 15,
    #[description = "Radar 72-hour QPE Accumulation Quality"]
    #[abbrev = "RadarAccumulationQualityIndex_72H"]
    #[unit = "non-dim"]
    RadarAccumulationQualityIndex72H = 16,
    #[description = "Gauge Influence Index for 1-hour QPE (2-hour latency)"]
    #[abbrev = "GaugeInflIndex_01H_Pass2"]
    #[unit = "non-dim"]
    GaugeInflIndex01HPass2 = 17,
    #[description = "Gauge Influence Index for 3-hour QPE (2-hour latency)"]
    #[abbrev = "GaugeInflIndex_03H_Pass2"]
    #[unit = "non-dim"]
    GaugeInflIndex03HPass2 = 18,
    #[description = "Gauge Influence Index for 6-hour QPE (2-hour latency)"]
    #[abbrev = "GaugeInflIndex_06H_Pass2"]
    #[unit = "non-dim"]
    GaugeInflIndex06HPass2 = 19,
    #[description = "Gauge Influence Index for 12-hour QPE (2-hour latency)"]
    #[abbrev = "GaugeInflIndex_12H_Pass2"]
    #[unit = "non-dim"]
    GaugeInflIndex12HPass2 = 20,
    #[description = "Gauge Influence Index for 24-hour QPE (2-hour latency)"]
    #[abbrev = "GaugeInflIndex_24H_Pass2"]
    #[unit = "non-dim"]
    GaugeInflIndex24HPass2 = 21,
    #[description = "Gauge Influence Index for 48-hour QPE (2-hour latency)"]
    #[abbrev = "GaugeInflIndex_48H_Pass2"]
    #[unit = "non-dim"]
    GaugeInflIndex48HPass2 = 22,
    #[description = "Gauge Influence Index for 72-hour QPE (2-hour latency)"]
    #[abbrev = "GaugeInflIndex_72H_Pass2"]
    #[unit = "non-dim"]
    GaugeInflIndex72HPass2 = 23,
    Missing = 255,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter)]
pub enum MRMSMultiRadarReflectivityProduct {
    #[description = "3D Reflectivity Mosaic - 33 CAPPIS (500-19000m)"]
    #[abbrev = "MergedReflectivityQC"]
    #[unit = "dBZ"]
    MergedReflectivityQC = 0,
    #[description = "3D RhoHV Mosaic - 33 CAPPIS (500-19000m)"]
    #[abbrev = "MergedRhoHV"]
    #[unit = "non-dim"]
    MergedRhoHV = 3,
    #[description = "3D Zdr Mosaic - 33 CAPPIS (500-19000m)"]
    #[abbrev = "MergedZdr"]
    #[unit = "dB"]
    MergedZdr = 4,
    Missing = 255,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter)]
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
    #[abbrev = "HeightLowLevelCompositeReflectivity"]
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
    #[description = "Layer Composite Reflectivity Mosaic (2-4.5km) (for ANC)"]
    #[abbrev = "LayerCompositeReflectivity_ANC"]
    #[unit = "dBZ"]
    LayerCompositeReflectivityANC = 9,
    #[description = "Base Reflectivity Hourly Maximum"]
    #[abbrev = "BREF_1HR_MAX"]
    #[unit = "dBZ"]
    BREF1HRMAX = 10,
    Missing = 255,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter)]
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

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter)]
pub enum MRMSFlashProduct {
    #[description = "FLASH QPE-CREST Unit Streamflow"]
    #[abbrev = "FLASH_CREST_MAXUNITSTREAMFLOW"]
    #[unit = "m^3/s/km^2"]
    FlashCrestMaxUnitStreamflow = 0,
    #[description = "FLASH QPE-CREST Streamflow"]
    #[abbrev = "FLASH_CREST_MAXSTREAMFLOW"]
    #[unit = "m^3/s"]
    FlashCrestMaxStreamflow = 1,
    #[description = "FLASH QPE-CREST Soil Saturation"]
    #[abbrev = "FLASH_CREST_MAXSOILSAT"]
    #[unit = "%"]
    FlashCrestMaxSoilSaturation = 2,
    #[description = "FLASH QPE-SAC Unit Streamflow"]
    #[abbrev = "FLASH_SAC_MAXUNITSTREAMFLOW"]
    #[unit = "m^3/s/km^2"]
    FlashSacMaxUnitStreamflow = 4,
    #[description = "FLASH QPE-SAC Streamflow"]
    #[abbrev = "FLASH_SAC_MAXSTREAMFLOW"]
    #[unit = "m^3/s"]
    FlashSacMaxStreamflow = 5,
    #[description = "FLASH QPE-SAC Soil Saturation"]
    #[abbrev = "FLASH_SAC_MAXSOILSAT"]
    #[unit = "%"]
    FlashSacMaxSoilSaturation = 6,
    #[description = "FLASH QPE Average Recurrence Interval 30-min"]
    #[abbrev = "FLASH_QPE_ARI30M"]
    #[unit = "years"]
    FlashQpeAri30M = 14,
    #[description = "FLASH QPE Average Recurrence Interval 01H"]
    #[abbrev = "FLASH_QPE_ARI01H"]
    #[unit = "years"]
    FlashQpeAri01H = 15,
    #[description = "FLASH QPE Average Recurrence Interval 03H"]
    #[abbrev = "FLASH_QPE_ARI03H"]
    #[unit = "years"]
    FlashQpeAri03H = 16,
    #[description = "FLASH QPE Average Recurrence Interval 06H"]
    #[abbrev = "FLASH_QPE_ARI06H"]
    #[unit = "years"]
    FlashQpeAri06H = 17,
    #[description = "FLASH QPE Average Recurrence Interval 12H"]
    #[abbrev = "FLASH_QPE_ARI12H"]
    #[unit = "years"]
    FlashQpeAri12H = 18,
    #[description = "FLASH QPE Average Recurrence Interval 24H"]
    #[abbrev = "FLASH_QPE_ARI24H"]
    #[unit = "years"]
    FlashQpeAri24H = 19,
    #[description = "FLASH QPE Average Recurrence Interval Maximum"]
    #[abbrev = "FLASH_QPE_ARIMAX"]
    #[unit = "years"]
    FlashQpeAriMax = 20,
    #[description = "FLASH QPE-to-FFG Ratio 01H"]
    #[abbrev = "FLASH_QPE_FFG01H"]
    #[unit = "non-dim"]
    FlashQpeFfg01H = 26,
    #[description = "FLASH QPE-to-FFG Ratio 03H"]
    #[abbrev = "FLASH_QPE_FFG03H"]
    #[unit = "non-dim"]
    FlashQpeFfg03H = 27,
    #[description = "FLASH QPE-to-FFG Ratio 06H"]
    #[abbrev = "FLASH_QPE_FFG06H"]
    #[unit = "non-dim"]
    FlashQpeFfg06H = 28,
    #[description = "FLASH QPE-to-FFG Ratio Maximum"]
    #[abbrev = "FLASH_QPE_FFGMAX"]
    #[unit = "non-dim"]
    FlashQpeFfgMax = 29,
    #[description = "FLASH QPE-Hydrophobic Unit Streamflow"]
    #[abbrev = "FLASH_HP_MAXUNITSTREAMFLOW"]
    #[unit = "m^3/s/km^2"]
    FlashHpMaxUnitStreamflow = 39,
    #[description = "FLASH QPE-Hydrophobic Streamflow"]
    #[abbrev = "FLASH_HP_MAXSTREAMFLOW"]
    #[unit = "m^3/s"]
    FlashHpMaxStreamflow = 40,
    Missing = 255,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter)]
pub enum MRMSConvectiveForecastProduct {
    #[description = "Likelihood of convection over the next 01H"]
    #[abbrev = "ANC_ConvectiveLikelihood"]
    #[unit = "non-dim"]
    ANCConvectiveLikelihood = 0,
    #[description = "01H reflectivity forecast"]
    #[abbrev = "ANC_FinalForecast"]
    #[unit = "dBZ"]
    ANCFinalForecast = 1,
    Missing = 255,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter)]
pub enum MRMSLevel3Product {
    #[description = "Level III High Resolution Enhanced Echo Top mosaic"]
    #[abbrev = "LVL3_HREET"]
    #[unit = "kft"]
    LVL3HREET = 0,
    #[description = "Level III High Resolution VIL mosaic"]
    #[abbrev = "LVL3_HighResVIL"]
    #[unit = "kg/m^2"]
    LVL3HighResVIL = 1,
    Missing = 255,
}

pub fn multiradar_parameter(category: u8, parameter: u8) -> Option<Parameter> {
    match category {
        2 => Some(Parameter::from(MRMSLightningProduct::from(parameter))),
        3 => Some(Parameter::from(MRMSConvectionProduct::from(parameter))),
        6 => Some(Parameter::from(MRMSPrecipitationProduct::from(parameter))),
        7 => Some(Parameter::from(MRMSModelProduct::from(parameter))),
        8 => Some(Parameter::from(MRMSQualityProduct::from(parameter))),
        9 => Some(Parameter::from(MRMSMultiRadarReflectivityProduct::from(
            parameter,
        ))),
        10 => Some(Parameter::from(MRMSCompositeReflectivityProduct::from(
            parameter,
        ))),
        11 => Some(Parameter::from(MRMSMergedReflectivityProduct::from(
            parameter,
        ))),
        12 => Some(Parameter::from(MRMSFlashProduct::from(parameter))),
        13 => Some(Parameter::from(MRMSConvectiveForecastProduct::from(
            parameter,
        ))),
        14 => Some(Parameter::from(MRMSLevel3Product::from(parameter))),
        _ => None,
    }
}

pub fn multiradar_category(category: u8) -> &'static str {
    match category {
        2 => "lightning",
        3 => "convection",
        6 => "precipitation",
        7 => "model",
        8 => "quality",
        9 => "3d reflectivity",
        10 => "composite reflectivity",
        11 => "merged reflectivity",
        12 => "flash",
        13 => "convective forecast",
        14 => "level 3",
        _ => "misc",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mrms_v12_2_parameters() {
        // NSSL UserTable_MRMS_v12.2 entries must resolve to real parameters, not Missing
        let cases = [
            (2u8, 1u8, "NLDN_CG_005min_AvgDensity", "flashes/km^2/min"),
            (2, 2, "NLDN_CG_015min_AvgDensity", "flashes/km^2/min"),
            (2, 3, "NLDN_CG_030min_AvgDensity", "flashes/km^2/min"),
            (3, 0, "MergedAzShear0to2kmAGL", "0.001/s"),
            (3, 6, "RotationTrack360min", "0.001/s"),
            (3, 46, "EchoTop_50", "km MSL"),
            (3, 54, "Reflectivity_-10C", "dBZ"),
            (6, 44, "SyntheticPrecipRateID", "flag"),
            (7, 3, "Model_0degC_Height", "m MSL"),
            (8, 0, "RadarQualityIndex", "non-dim"),
            (8, 8, "SeamlessHSR", "dBZ"),
            (8, 9, "SeamlessHSRHeight", "km AGL"),
            (8, 23, "GaugeInflIndex_72H_Pass2", "non-dim"),
            (9, 0, "MergedReflectivityQC", "dBZ"),
            (10, 3, "HeightLowLevelCompositeReflectivity", "m MSL"),
            (10, 9, "LayerCompositeReflectivity_ANC", "dBZ"),
            (12, 0, "FLASH_CREST_MAXUNITSTREAMFLOW", "m^3/s/km^2"),
            (12, 20, "FLASH_QPE_ARIMAX", "years"),
            (13, 1, "ANC_FinalForecast", "dBZ"),
            (14, 0, "LVL3_HREET", "kft"),
        ];
        for (category, number, abbrev, unit) in cases {
            let param = multiradar_parameter(category, number).unwrap();
            assert_eq!(param.abbrev, abbrev, "parameter ({category}, {number})");
            assert_eq!(param.unit, unit, "parameter ({category}, {number})");
            assert_ne!(param.name, "missing", "parameter ({category}, {number})");
        }
    }

    #[test]
    fn test_mrms_categories() {
        for category in [2u8, 3, 6, 7, 8, 9, 10, 11, 12, 13, 14] {
            assert!(multiradar_parameter(category, 0).is_some());
            assert_ne!(multiradar_category(category), "misc");
        }
        assert!(multiradar_parameter(4, 0).is_none());
        assert_eq!(multiradar_category(4), "misc");
    }
}
