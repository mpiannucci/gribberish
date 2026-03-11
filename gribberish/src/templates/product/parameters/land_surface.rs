use gribberish_macros::{DisplayDescription, FromValue, ToParameter};
use gribberish_types::Parameter;

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter)]
pub enum VegetationProduct {
    #[description = "land cover"]
    #[abbrev = "LAND"]
    #[unit = "proportion"]
    LandCover = 0,
    #[description = "surface roughness"]
    #[abbrev = "SFCR"]
    #[unit = "m"]
    SurfaceRoughness = 1,
    #[description = "soil temperature"]
    #[abbrev = "TSOIL"]
    #[unit = "K"]
    SoilTemperature = 2,
    #[description = "vegetation"]
    #[abbrev = "VEG"]
    #[unit = "%"]
    Vegetation = 4,
    #[description = "water runoff"]
    #[abbrev = "WATR"]
    #[unit = "kgm-2"]
    WaterRunoff = 5,
    #[description = "ground heat flux"]
    #[abbrev = "GFLUX"]
    #[unit = "Wm-2"]
    GroundHeatFlux = 10,
    #[description = "plant canopy surface water"]
    #[abbrev = "CNWAT"]
    #[unit = "kgm-2"]
    PlantCanopySurfaceWater = 13,
    #[description = "volumetric soil moisture"]
    #[abbrev = "VSW"]
    #[unit = "m3m-3"]
    VolumetricSoilMoisture = 25,
    #[description = "runoff water equivalent"]
    #[abbrev = "ROWE"]
    #[unit = "kgm-2"]
    RunoffWaterEquivalent = 42,
    #[description = "soil moisture"]
    #[abbrev = "SM"]
    #[unit = "kgm-3"]
    SoilMoisture = 22,
    #[description = "volumetric soil moisture content (NCEP)"]
    #[abbrev = "SOILW"]
    #[unit = "proportion"]
    VolumetricSoilMoistureContentNcep = 192,
    #[description = "ground heat flux (NCEP)"]
    #[abbrev = "GFLUX"]
    #[unit = "Wm-2"]
    GroundHeatFluxNcep = 193,
    #[description = "moisture availability (NCEP)"]
    #[abbrev = "MSTAV"]
    #[unit = "%"]
    MoistureAvailabilityNcep = 194,
    #[description = "plant canopy surface water (NCEP)"]
    #[abbrev = "CNWAT"]
    #[unit = "kgm-2"]
    PlantCanopySurfaceWaterNcep = 196,
    #[description = "vegetation type"]
    #[abbrev = "VGTYP"]
    #[unit = "integer"]
    VegetationType = 198,
    #[description = "wilting point"]
    #[abbrev = "WILT"]
    #[unit = "fraction"]
    WiltingPoint = 201,
    #[description = "seasonally minimum green vegetation fraction"]
    #[abbrev = "VEGMIN"]
    #[unit = "%"]
    SeasonalMinGreenVegetationFraction = 231,
    #[description = "seasonally maximum green vegetation fraction"]
    #[abbrev = "VEGMAX"]
    #[unit = "%"]
    SeasonalMaxGreenVegetationFraction = 232,
    Missing = 255,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter)]
pub enum SoilProduct {
    #[description = "soil type"]
    #[abbrev = "SOTYP"]
    #[unit = "seetable4.213"]
    SoilType = 0,
    #[description = "upper layer soil temperature"]
    #[abbrev = "TSOIL"]
    #[unit = "K"]
    UpperLayerSoilTemperature = 1,
    #[description = "upper layer soil moisture"]
    #[abbrev = "SOILM"]
    #[unit = "kgm-3"]
    UpperLayerSoilMoisture = 2,
    #[description = "lower layer soil moisture"]
    #[abbrev = "SOILM"]
    #[unit = "kgm-3"]
    LowerLayerSoilMoisture = 3,
    #[description = "bottom layer soil temperature"]
    #[abbrev = "TSOIL"]
    #[unit = "K"]
    BottomLayerSoilTemperature = 4,
    #[description = "liquid volumetric soil moisture"]
    #[abbrev = "SOILM"]
    #[unit = "proportion"]
    LiquidVolumetricSoilMoisture = 10,
    #[description = "soil temperature"]
    #[abbrev = "TSOIL"]
    #[unit = "K"]
    SoilTemperatureWmo = 18,
    #[description = "volumetric wilting point"]
    #[abbrev = "WILT"]
    #[unit = "proportion"]
    VolumetricWiltingPoint = 27,
    #[description = "liquid volumetric soil moisture (non frozen)"]
    #[abbrev = "SOILL"]
    #[unit = "proportion"]
    LiquidVolumetricSoilMoistureNonFrozen = 192,
    #[description = "field capacity"]
    #[abbrev = "FLDCP"]
    #[unit = "proportion"]
    FieldCapacity = 200,
    #[description = "field capacity"]
    #[abbrev = "FLDCP"]
    #[unit = "proportion"]
    FieldCapacityNcep = 203,
    Missing = 255,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter)]
pub enum FireWeatherProduct {
    #[description = "fire outlook"]
    #[abbrev = "FIREOLK"]
    #[unit = "nondim"]
    FireOutlook = 0,
    #[description = "fire outlook due to dry thunderstorm"]
    #[abbrev = "DRYTSTM"]
    #[unit = "nondim"]
    FireOutlookDryThunderstorm = 1,
    #[description = "haines index"]
    #[abbrev = "HINDEX"]
    #[unit = "nondim"]
    HainesIndex = 2,
    Missing = 255,
}

pub fn land_surface_parameter(category: u8, parameter: u8) -> Option<Parameter> {
    match category {
        0 => Some(Parameter::from(VegetationProduct::from(parameter))),
        3 => Some(Parameter::from(SoilProduct::from(parameter))),
        4 => Some(Parameter::from(FireWeatherProduct::from(parameter))),
        _ => None,
    }
}

pub fn land_surface_category(category: u8) -> &'static str {
    match category {
        0 => "vegetation/biomass",
        3 => "soil",
        4 => "fire weather",
        _ => "other",
    }
}
