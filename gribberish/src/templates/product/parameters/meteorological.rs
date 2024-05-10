use gribberish_macros::{DisplayDescription, FromValue, ToParameter};
use gribberish_types::Parameter;

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter)]
pub enum TemperatureProduct {
    #[abbrev = "TMP"]
    #[unit = "K"]
    Temperature = 0,
    #[description = "virtual temperature"]
    #[abbrev = "VTMP"]
    #[unit = "K"]
    VirtualTemperature = 1,
    #[description = "potential temperature"]
    #[abbrev = "POT"]
    #[unit = "K"]
    PotentialTemperature = 2,
    #[description = "pseudo-adiabatic potential temperature"]
    #[abbrev = "EPOT"]
    #[unit = "K"]
    PseudoAdiabaticPotentialTemperature = 3,
    #[description = "maximum temperature"]
    #[abbrev = "TMAX"]
    #[unit = "K"]
    MaximumTemperature = 4,
    #[description = "minimum temperature"]
    #[abbrev = "TMIN"]
    #[unit = "K"]
    MinimumTemperature = 5,
    #[description = "dewpoint temperature"]
    #[abbrev = "DPT"]
    #[unit = "K"]
    DewpointTemperature = 6,
    #[description = "dewpoint depression"]
    #[abbrev = "DEPR"]
    #[unit = "K"]
    DewpointDepression = 7,
    #[description = "lapse rate"]
    #[abbrev = "LAPR"]
    #[unit = "Km-1"]
    LapseRate = 8,
    #[description = "heat index"]
    #[abbrev = "HEATX"]
    #[unit = "K"]
    HeatIndex = 12,
    #[description = "wind chill factor"]
    #[abbrev = "WCF"]
    #[unit = "K"]
    WindChillFactor = 13,
    #[description = "apparent temperature"]
    #[abbrev = "APTMP"]
    #[unit = "K"]
    ApparentTemperature = 21,
    Missing = 255,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter)]
pub enum MoistureProduct {
    #[description = "specific humidity"]
    #[abbrev = "SPFH"]
    #[unit = "kgkg-1"]
    SpecificHumidity = 0,
    #[description = "relative humidity"]
    #[abbrev = "RH"]
    #[unit = "%"]
    RelativeHumidity = 1,
    #[description = "humidity mixing ratio"]
    #[abbrev = "MIXR"]
    #[unit = "kgkg-1"]
    HumidityMixingRatio = 2,
    #[description = "precipitable water"]
    #[abbrev = "PWAT"]
    #[unit = "kgm-2"]
    PrecipitableWater = 3,
    #[abbrev = "EVP"]
    #[unit = "kgm-2"]
    Evaporation = 4,
    #[description = "precipitation rate"]
    #[abbrev = "PRATE"]
    #[unit = "kgm-2s-1"]
    PrecipitationRate = 7,
    #[description = "total precipitation"]
    #[abbrev = "APCP"]
    #[unit = "kgm-2"]
    TotalPrecipitation = 8,
    #[description = "convective precipitation"]
    #[abbrev = "ACPCP"]
    #[unit = "gm-2"]
    ConvectivePrecipitation = 10,
    #[description = "snow depth"]
    #[abbrev = "SNOD"]
    #[unit = "m"]
    SnowDepth = 11,
    #[description = "water Equivalent of accumulated snow depth"]
    #[abbrev = "WEASD"]
    #[unit = "kgm-2"]
    WaterEquavalentSnowDepth = 13,
    #[description = "total snowfall"]
    #[abbrev = "ASNOW"]
    #[unit = "m"]
    TotalSnowfall = 29,
    #[abbrev = "HAIL"]
    #[unit = "m"]
    Hail=31,
    #[description = "categorical rain"]
    #[abbrev = "CRAIN"]
    #[unit = "BOOL"]
    CategoricalRain = 33,
    #[description = "categorical freezing rain"]
    #[abbrev = "CFRZR"]
    #[unit = "BOOL"]
    CategoricalFreezingRain = 34,
    #[description = "categorical ice pellets"]
    #[abbrev = "CICEP"]
    #[unit = "BOOL"]
    CategoricalIcePellets = 35,
    #[description = "categorical snow"]
    #[abbrev = "CSNOW"]
    #[unit = "BOOL"]
    CategoricalSnow = 36,
    #[description = "percent frozen precipitation"]
    #[abbrev = "CPOFP"]
    #[unit = "%"]
    PercentFrozenPrecipitation = 39,
    #[description = "snow cover"]
    #[abbrev = "SNOWC"]
    #[unit = "%"]
    SnowCover = 42,
    #[description = "total column integrated graupel"]
    #[abbrev = "TCOLG"]
    #[unit = "kgm-2"]
    TotalColumnIntegratedGraupel = 74,
    #[description = "freezing rain"]
    #[abbrev = "FRZR"]
    #[unit = "kgm-2"]
    FreezingRain = 225,
    #[description = "frozen rain"]
    #[abbrev = "FROZR"]
    #[unit = "kgm-2"]
    FrozenRain = 227,
    Missing = 255,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter)]
pub enum MomentumProduct {
    #[description = "wind direction"]
    #[abbrev = "WDIR"]
    #[unit = "degrees"]
    WindDirection = 0,
    #[description = "wind speed"]
    #[abbrev = "WIND"]
    #[unit = "ms-1"]
    WindSpeed = 1,
    #[description = "u-component of wind speed"]
    #[abbrev = "UGRD"]
    #[unit = "ms-1"]
    UComponentWindSpeed = 2,
    #[description = "v-component of wind speed"]
    #[abbrev = "VGRD"]
    #[unit = "ms-1"]
    VComponentWindSpeed = 3,
    #[abbrev = "RELV"]
    #[unit = "s-1"]
    RelativeVorticity = 12,
    #[description = "Maximum wind speed"]
    #[abbrev = "MAXGUST"]
    #[unit = "ms-1"]
    MaximumWindSpeed = 21,
    #[description = "wind gust speed"]
    #[abbrev = "GUST"]
    #[unit = "ms-1"]
    WindGust = 22,
    #[description = "u-component of wind gust"]
    #[abbrev = "UGUST"]
    #[unit = "ms-1"]
    UComponentWindGust = 23,
    #[description = "v-component of wind gust"]
    #[abbrev = "VGUST"]
    #[unit = "ms-1"]
    VComponentWindGust = 24,
    #[description = "wind fetch"]
    #[abbrev = "WINDF"]
    #[unit = "m"]
    WindFetch = 33,
    #[description = "u-component of storm motion"]
    #[abbrev = "UTSM"]
    #[unit = "ms-1"]
    UComponentStormMotion = 27,
    #[description = "v-component of storm motion"]
    #[abbrev = "VSTM"]
    #[unit = "ms-1"]
    VComponentStormMotion = 28,
    #[description = "u component of hourly maximum 10m wind speed"]
    #[abbrev = "MAXUW"]
    #[unit = "ms-1"]
    UComponentHourlyMaximumWindSpeed = 222,
    #[description = "v component of hourly maximum 10m wind speed"]
    #[abbrev = "MAXVW"]
    #[unit = "ms-1"]
    VComponentHourlyMaximumWindSpeed = 223,
    #[description = "tropical wind direction"]
    #[abbrev = "TPWDIR"]
    #[unit = "degrees"]
    TropicalWindDirection = 231,
    #[description = "tropical wind speed"]
    #[abbrev = "TPWSPD"]
    #[unit = "ms-1"]
    TropicalWindSpeed = 232,
    Missing = 255,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter)]
pub enum CloudProduct {
    #[description = "total cloud cover"]
    #[abbrev = "TCDC"]
    #[unit = "%"]
    TotalCloudCover = 1,
    #[description = "convective cloud cover"]
    #[abbrev = "CDCON"]
    #[unit = "%"]
    ConvectiveCloudCover = 2,
    #[description = "low cloud cover"]
    #[abbrev = "LCDC"]
    #[unit = "%"]
    LowCloudCover = 3,
    #[description = "middle cloud cover"]
    #[abbrev = "MCDC"]
    #[unit = "%"]
    MediumCloudCover = 4,
    #[description = "high cloud cover"]
    #[abbrev = "HCDC"]
    #[unit = "%"]
    HighCloudCover = 5,
    Missing = 255,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter)]
pub enum MassProduct {
    #[abbrev = "PRES"]
    #[unit = "pa"]
    Pressure = 0,
    #[description = "pressure reduced to MSL"]
    #[abbrev = "PRMSL"]
    #[unit = "pa"]
    PressureReducedMSL = 1,
    #[description = "pressure tendency"]
    #[abbrev = "PTEND"]
    #[unit = "pas-1"]
    PressureTendency = 2,
    #[description = "geopotential height"]
    #[abbrev = "HGT"]
    #[unit = "gpm"]
    GeopotentialHeight = 5,
    #[description = "mslp (eta model reduction)"]
    #[abbrev = "MSLET"]
    #[unit = "pa"]
    MSLP = 192,
    Missing = 255,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter)]
pub enum RadarProduct {
    #[description = "base spectrum width"]
    #[abbrev = "BSWID"]
    #[unit = "ms-1"]
    BaseSpectrumWidth = 0,
    #[description = "base reflectivity"]
    #[abbrev = "BREF"]
    #[unit = "dB"]
    BaseReflectivity = 1,
    #[description = "layer maximum base reflectivity"]
    #[abbrev = "LMAXBR"]
    #[unit = "dB"]
    LayerMaximumBaseReflectivity = 4,
    #[description = "precipitation"]
    #[abbrev = "PREC"]
    #[unit = "kgm-2"]
    Precipitation = 5,
    Missing = 255,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter)]
pub enum ForecastRadarImagery {
    #[description = "echo top"]
    #[abbrev = "RETOP"]
    #[unit = "m"]
    EchoTop = 3,
    #[description = "reflectivity"]
    #[abbrev = "REFD"]
    #[unit = "dB"]
    Reflectivity = 195,
    #[description = "composite reflectivity"]
    #[abbrev = "REFC"]
    #[unit = "dB"]
    CompositeReflectivity = 196,
    Missing = 255,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter)]
pub enum Electromagnetics {
    #[abbrev = "LTNGSD"]
    #[unit = "m-2 s-1"]
    LightingStrikeDensity = 0,
    #[description = "lightning potential index"]
    #[abbrev = "LTPINX"]
    #[unit = "J kg-1"]
    LightingPotentialIndex = 1,
    #[description = "cloud-to-ground lightning flash density"]
    #[abbrev = "CDGDLTFD"]
    #[unit = "km-2 day-1"]
    CloudToGroundLightingFlashDensity = 2,
    #[description = "cloud-to-cloud lightning flash density"]
    #[abbrev = "CDCDLTFD"]
    #[unit = "km-2 day-1"]
    CloudToCloudLightingFlashDensity = 3,
    #[description = "total lightning flash density"]
    #[abbrev = "TLGTFD"]
    #[unit = "km-2 day-1"]
    TotalLightningFlashDensity = 4,
    #[abbrev = "LTNG"]
    #[unit = "nondim"]
    Lightning = 192,
    Missing = 255,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter)]
pub enum PhysicalAtmosphericProperties {
    #[description = "visibility"]
    #[abbrev = "vis"]
    #[unit = "m"]
    Visibility = 0,
    Missing = 255,
}

pub fn meteorological_parameter(category: u8, parameter: u8) -> Option<Parameter> {
    match category {
        0 => Some(Parameter::from(TemperatureProduct::from(parameter))),
        1 => Some(Parameter::from(MoistureProduct::from(parameter))),
        2 => Some(Parameter::from(MomentumProduct::from(parameter))),
        3 => Some(Parameter::from(MassProduct::from(parameter))),
        6 => Some(Parameter::from(CloudProduct::from(parameter))),
        15 => Some(Parameter::from(RadarProduct::from(parameter))),
        16 => Some(Parameter::from(ForecastRadarImagery::from(parameter))),
        17 => Some(Parameter::from(Electromagnetics::from(parameter))),
        19 => Some(Parameter::from(PhysicalAtmosphericProperties::from(
            parameter,
        ))),
        _ => None,
    }
}

pub fn meteorological_category(category: u8) -> &'static str {
    match category {
        0 => "temperature",
        1 => "moisture",
        2 => "momentum",
        3 => "mass",
        6 => "cloud",
        15 => "radar",
        16 => "forecast radar imagery",
        17 => "electromagnetics",
        19 => "physical atmospheric properties",
        _ => "other",
    }
}
