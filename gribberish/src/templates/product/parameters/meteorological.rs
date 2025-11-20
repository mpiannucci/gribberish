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
    ShortWaveRadiation = 4,
    #[description = "vertical velocity (pressure)"]
    #[abbrev = "VVEL"]
    #[unit = "Pas-1"]
    VerticalVelocityPressure = 8,
    #[description = "vertical velocity (geometric)"]
    #[abbrev = "DZDT"]
    #[unit = "ms-1"]
    VerticalVelocityGeometric = 9,
    #[description = "absolute vorticity"]
    #[abbrev = "ABSV"]
    #[unit = "s-1"]
    AbsoluteVorticity = 10,
    #[abbrev = "SWRD"]
    #[unit = "Wm-2"]
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
pub enum ShortWaveRadiationProduct {
    #[description = "net shortwave radiation flux surface"]
    #[abbrev = "nswrs"]
    #[unit = "Wm-2"]
    NetShortwaveRadiationFluxSurface = 0,
    #[description = "net shortwave radiation flux top of atmosphere"]
    #[abbrev = "nswrt"]
    #[unit = "Wm-2"]
    NetShortwaveRadiationFluxTop = 1,
    #[description = "shortwave radiation flux"]
    #[abbrev = "swavr"]
    #[unit = "Wm-2"]
    ShortwaveRadiationFlux = 2,
    #[description = "global radiation flux"]
    #[abbrev = "grad"]
    #[unit = "Wm-2"]
    GlobalRadiationFlux = 3,
    #[description = "brightness temperature"]
    #[abbrev = "brtmp"]
    #[unit = "K"]
    BrightnessTemperature = 4,
    #[description = "radiance with respect to wave number"]
    #[abbrev = "lwrad"]
    #[unit = "Wm-1sr-1"]
    RadianceFromWaveNumber = 5,
    #[description = "radiance with respect to wavelength"]
    #[abbrev = "swrad"]
    #[unit = "Wm-3sr-1"]
    RadianceFromWavelength = 6,
    #[description = "downward shortwave radiation flux"]
    #[abbrev = "dswrf"]
    #[unit = "Wm-2"]
    DownwardShortwaveRadiationFlux = 7,
    #[description = "upward shortwave radiation flux"]
    #[abbrev = "uswrf"]
    #[unit = "Wm-2"]
    UpwardShortwaveRadiationFlux = 8,
    #[description = "net short wave radiation flux"]
    #[abbrev = "nswrf"]
    #[unit = "Wm-2"]
    NetShortWaveRadiationFlux = 9,
    #[description = "photosynthetically active radiation"]
    #[abbrev = "photar"]
    #[unit = "Wm-2"]
    PhotosyntheticallyActiveRadiation = 10,
    #[description = "net short wave radiation flux, clear sky"]
    #[abbrev = "nswrfcs"]
    #[unit = "Wm-2"]
    NetShortWaveRadiationFluxClearSky = 11,
    #[description = "downward uv radiation"]
    #[abbrev = "dwuvr"]
    #[unit = "Wm-2"]
    DownwardUVRadiation = 12,
    #[description = "direct short wave radiation flux"]
    #[abbrev = "dswrflx"]
    #[unit = "Wm-2"]
    DirectShortWaveRadiationFlux = 13,
    #[description = "diffuse short wave radiation flux"]
    #[abbrev = "difswrf"]
    #[unit = "Wm-2"]
    DiffuseShortWaveRadiationFlux = 14,
    #[description = "upward uv radiation emitted/reflected from the earth's surface"]
    #[abbrev = "uvvearth"]
    #[unit = "Wm-2"]
    UpwardUVRadiationEmittedReflectedFromTheEarth = 15,
    #[description = "uv index clear sky"]
    #[abbrev = "uviucs"]
    #[unit = "numeric"]
    UVIndexClearSky = 50,
    #[description = "uv index"]
    #[abbrev = "uvi"]
    #[unit = "numeric"]
    UVIndex = 51,
    #[description = "downward short wave radiation flux clear sky"]
    #[abbrev = "dswrfcs"]
    #[unit = "Wm-2"]
    DownwardShortWaveRadiationFluxClearSky = 52,
    #[description = "upward short wave radiation flux clear sky"]
    #[abbrev = "uswrfcs"]
    #[unit = "Wm-2"]
    UpwardShortWaveRadiationFluxClearSky = 53,
    #[description = "direct normal short wave radiation flux,"]
    #[abbrev = "dnswrflx"]
    #[unit = "Wm-2"]
    DirectNormalShortWaveRadiationFlux = 54,
    #[description = "uv visible albedo for diffuse radiation"]
    #[abbrev = "uvalbdif"]
    #[unit = "%"]
    UVVisibleAlbedoForDiffuseRadiation = 55,
    #[description = "uv visible albedo for direct radiation"]
    #[abbrev = "uvalbdir"]
    #[unit = "%"]
    UVVisibleAlbedoForDirectRadiation = 56,
    #[description = "uv visible albedo for direct radiation, geometric component"]
    #[abbrev = "ubalbdirg"]
    #[unit = "%"]
    UVVisibleAlbedoForDirectRadiationGeometricComponent = 57,
    #[description = "uv visible albedo for direct radiation, isotropic component"]
    #[abbrev = "uvalbdiri"]
    #[unit = "%"]
    UVVisibleAlbedoForDirectRadiationIsotropicComponent = 58,
    #[description = "uv visible albedo for direct radiation, volumetric component"]
    #[abbrev = "uvbdirv"]
    #[unit = "%"]
    UVVisibleAlbedoForDirectRadiationVolumetricComponent = 59,
    #[description = "photosynthetically active radiation flux, clear sky"]
    #[abbrev = "phoarfcs"]
    #[unit = "Wm-2"]
    PhotosyntheticallyActiveRadiationFluxClearSky = 60,
    #[description = "direct short wave radiation flux, clear sky"]
    #[abbrev = "dswrflxcs"]
    #[unit = "Wm-2"]
    DirectShortWaveRadiationFluxClearSky = 61,
    #[description = "downward short wave radiation flux"]
    #[abbrev = "dswrf"]
    #[unit = "Wm-2"]
    DownwardShortWaveRadiationFlux = 192,
    #[description = "upward short wave radiation flux"]
    #[abbrev = "uswrf"]
    #[unit = "Wm-2"]
    UpwardShortWaveRadiationFlux = 193,
    #[description = "uv b downward solar flux"]
    #[abbrev = "duvb"]
    #[unit = "Wm-2"]
    UVBDownwardSolarFlux = 194,
    #[description = "clear sky uv b downward solar flux"]
    #[abbrev = "cduvb"]
    #[unit = "Wm-2"]
    ClearSkyUVBDownwardSolarFlux = 195,
    #[description = "clear sky downward solar flux"]
    #[abbrev = "csdsf"]
    #[unit = "Wm-2"]
    ClearSkyDownwardSolarFlux = 196,
    #[description = "solar radiative heating rate"]
    #[abbrev = "swhr"]
    #[unit = "Ks-1"]
    SolarRadiativeHeatingRate = 197,
    #[description = "clear sky upward solar flux"]
    #[abbrev = "csusf"]
    #[unit = "Wm-2"]
    ClearSkyUpwardSolarFlux = 198,
    #[description = "cloud forcing net solar flux"]
    #[abbrev = "cfnsf"]
    #[unit = "Wm-2"]
    CloudForcingNetSolarFlux = 199,
    #[description = "visible beam downward solar flux"]
    #[abbrev = "vbdsf"]
    #[unit = "Wm-2"]
    VisibleBeamDownwardSolarFlux = 200,
    #[description = "visible diffuse downward solar flux"]
    #[abbrev = "vddsf"]
    #[unit = "Wm-2"]
    VisibleDiffuseDownwardSolarFlux = 201,
    #[description = "near ir beam downward solar flux"]
    #[abbrev = "nbdsf"]
    #[unit = "Wm-2"]
    NearIrBeamDownwardSolarFlux = 202,
    #[description = "near ir diffuse downward solar flux"]
    #[abbrev = "nddsf"]
    #[unit = "Wm-2"]
    NearIrDiffuseDownwardSolarFlux = 203,
    #[description = "downward total radiation flux"]
    #[abbrev = "dtrf"]
    #[unit = "Wm-2"]
    DownwardTotalRadiationFlux = 204,
    #[description = "upward total radiation flux"]
    #[abbrev = "utrf"]
    #[unit = "Wm-2"]
    UpwardTotalRadiationFlux = 205,
    #[description = "diffuse short wave radiation flux, clear sky"]
    #[abbrev = "dfswrflxcs"]
    #[unit = "Wm-2"]
    DiffuseShortWaveRadiationFluxClearSky = 206,
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
pub enum ThermodynamicStabilityProduct {
    #[description = "parcel lifted index"]
    #[abbrev = "PLI"]
    #[unit = "K"]
    ParcelLiftedIndex = 0,
    #[description = "best lifted index"]
    #[abbrev = "BLI"]
    #[unit = "K"]
    BestLiftedIndex = 1,
    #[description = "k index"]
    #[abbrev = "KX"]
    #[unit = "K"]
    KIndex = 2,
    #[description = "ko index"]
    #[abbrev = "KOX"]
    #[unit = "K"]
    KOIndex = 3,
    #[description = "total totals index"]
    #[abbrev = "TOTALX"]
    #[unit = "K"]
    TotalTotalsIndex = 4,
    #[description = "sweat index"]
    #[abbrev = "SX"]
    #[unit = "numeric"]
    SweatIndex = 5,
    #[description = "convective available potential energy"]
    #[abbrev = "CAPE"]
    #[unit = "Jkg-1"]
    ConvectiveAvailablePotentialEnergy = 6,
    #[description = "convective inhibition"]
    #[abbrev = "CIN"]
    #[unit = "Jkg-1"]
    ConvectiveInhibition = 7,
    #[description = "storm relative helicity"]
    #[abbrev = "HLCY"]
    #[unit = "m2s-2"]
    StormRelativeHelicity = 8,
    #[description = "energy helicity index"]
    #[abbrev = "EHLX"]
    #[unit = "numeric"]
    EnergyHelicityIndex = 9,
    #[description = "surface lifted index"]
    #[abbrev = "LFTX"]
    #[unit = "K"]
    SurfaceLiftedIndex = 10,
    #[description = "best (4 layer) lifted index"]
    #[abbrev = "4LFTX"]
    #[unit = "K"]
    Best4LayerLiftedIndex = 11,
    #[description = "richardson number"]
    #[abbrev = "RI"]
    #[unit = "numeric"]
    RichardsonNumber = 12,
    Missing = 255,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter)]
pub enum PhysicalAtmosphericProperties {
    #[description = "visibility"]
    #[abbrev = "VIS"]
    #[unit = "m"]
    Visibility = 0,
    Missing = 255,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter)]
pub enum LongWaveRadiationProduct {
    #[description = "net long-wave radiation flux"]
    #[abbrev = "NLWRS"]
    #[unit = "Wm-2"]
    NetLongWaveRadiationFlux = 0,
    #[description = "downward long-wave radiation flux"]
    #[abbrev = "DLWRF"]
    #[unit = "Wm-2"]
    DownwardLongWaveRadiationFlux = 3,
    #[description = "upward long-wave radiation flux"]
    #[abbrev = "ULWRF"]
    #[unit = "Wm-2"]
    UpwardLongWaveRadiationFlux = 4,
    Missing = 255,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter)]
pub enum TraceGasesProduct {
    #[description = "total ozone"]
    #[abbrev = "TOZNE"]
    #[unit = "DU"]
    TotalOzone = 0,
    #[description = "ozone mixing ratio"]
    #[abbrev = "O3MR"]
    #[unit = "kgkg-1"]
    OzoneMixingRatio = 1,
    #[description = "total column integrated ozone"]
    #[abbrev = "TCIOZ"]
    #[unit = "DU"]
    TotalColumnIntegratedOzone = 2,
    #[description = "ozone mixing ratio"]
    #[abbrev = "O3MR"]
    #[unit = "kgkg-1"]
    OzoneMixingRatio192 = 192,
    #[description = "ozone concentration"]
    #[abbrev = "OZCON"]
    #[unit = "ppb"]
    OzoneConcentration = 193,
    #[description = "categorical ozone concentration"]
    #[abbrev = "OZCAT"]
    #[unit = "nondim"]
    CategoricalOzoneConcentration = 194,
    Missing = 255,
}

pub fn meteorological_parameter(category: u8, parameter: u8) -> Option<Parameter> {
    match category {
        0 => Some(Parameter::from(TemperatureProduct::from(parameter))),
        1 => Some(Parameter::from(MoistureProduct::from(parameter))),
        2 => Some(Parameter::from(MomentumProduct::from(parameter))),
        3 => Some(Parameter::from(MassProduct::from(parameter))),
        4 => Some(Parameter::from(ShortWaveRadiationProduct::from(parameter))),
        5 => Some(Parameter::from(LongWaveRadiationProduct::from(parameter))),
        6 => Some(Parameter::from(CloudProduct::from(parameter))),
        7 => Some(Parameter::from(ThermodynamicStabilityProduct::from(parameter))),
        14 => Some(Parameter::from(TraceGasesProduct::from(parameter))),
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
        4 => "short wave radiation",
        5 => "long wave radiation",
        6 => "cloud",
        7 => "thermodynamic stability",
        14 => "trace gases",
        15 => "radar",
        16 => "forecast radar imagery",
        17 => "electromagnetics",
        19 => "physical atmospheric properties",
        _ => "other",
    }
}
