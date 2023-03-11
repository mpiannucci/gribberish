use chrono::Duration;
use gribberish_macros::{DisplayDescription, FromValue, ToParameter};
use gribberish_types::Parameter;

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue)]
pub enum ClusteringMethod {
    #[description = "anomoly correlation"]
    AnomolyCorrelation = 0,
    #[description = "root mean square"]
    RMS = 1,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, Clone, DisplayDescription, FromValue, ToParameter)]
pub enum FixedSurfaceType {
    #[description = "ground or water surface"]
    GroundOrWater = 1,
    #[description = "cloud base level"]
    CloudBase = 2,
    #[description = "cloud tops level"]
    CloudTop = 3,
    #[description = "maximum wind level"]
    MaximumWindLevel = 6,
    #[description = "tropopause"]
    Tropopause = 7,
    #[description = "sea bottom"]
    SeaBottom = 9,
    #[description = "entire atmosphere"]
    EntireAtmosphere = 10,
    #[description = "isothermal level"]
    IsothermalLevel = 20,
    #[description = "mean sea level"]
    MeanSeaLevel = 101,
    #[description = "specific altitude above mean sea level"]
    SpecificAltitudeAboveMeanSeaLevel = 102,
    #[description = "specific height level above ground"]
    SpecifiedHeightLevelAboveGround = 103,
    #[description = "sigma level"]
    SigmaLevel = 104,
    #[description = "hybrid level"]
    HybridLevel = 105,
    #[description = "eta level"]
    EtaLevel = 111,
    #[description = "snow level"]
    SnowLevel = 114,
    #[description = "sigma height level"]
    SigmaHeightLevel = 115,
    #[description = "generalized vertical height coordinate"]
    GeneralizedVerticalHeightCoordinate = 150,
    #[description = "depth below sea level"]
    DepthBelowSeaLevel = 160,
    #[description = "depth below water surface"]
    DepthBelowWaterSurface = 161,
    #[description = "mixing layer"]
    MixingLayer = 166,
    #[description = "entire atmosphere as a single layer"]
    EntireAtmosphereAsSingleLayer = 200,
    #[description = "entire ocean as a single layer"]
    EntireOceanAsSingleLayer = 201,
    #[description = "ordered Sequence of Data"]
    OrderedSequence = 241,
    #[description = "missing"]
    Missing = 255,
}

#[repr(u8)]
#[derive(Clone, Eq, PartialEq, Debug, DisplayDescription, FromValue)]
pub enum GeneratingProcess {
    Analysis = 0,
    Initialization = 1,
    Forecast = 2,
    #[description = "bias corrected forecast"]
    BiasCorrectedForecast = 3,
    #[description = "ensemble forecast"]
    EnsembleForecast = 4,
    #[description = "probability forecast"]
    ProbabilityForecast = 5,
    #[description = "forecast error"]
    ForecastError = 6,
    #[description = "analysis error"]
    AnalysisError = 7,
    Observation = 8,
    Climatological = 9,
    #[description = "probability weighted forecast"]
    ProbabilityWeightedForecast = 10,
    #[description = "bias corrected ensemble forecast"]
    BiasCorrectedEnsembleForecast = 11,
    #[description = "post-processed analysis"]
    PostProcessedAnalysis = 12,
    #[description = "post-processed forecast"]
    PostProcessedForecast = 13,
    Nowcast = 14,
    Hindcast = 15,
    #[description = "physical retrieval"]
    PhysicalRetrieval = 16,
    #[description = "regression analysis"]
    RegressionAnalysis = 17,
    #[description = "difference between two forecasts"]
    DifferenceBetweenTwoForecasts = 18,
    #[description = "forecast confidence indicator"]
    ForecastConfidenceIndicator = 192,
    #[description = "probability matched mean"]
    ProbabilityMatchedMean = 193,
    #[description = "neighborhood probability"]
    NeighborhoodProbability = 194,
    #[description = "bias corrected downscaled ensemble forecast"]
    BiasCorrectedDownscaledEnsembleForecast = 195,
    #[description = "perturbed analysis for ensemble initialization"]
    PerturbedAnalysisForEnsembleInitialization = 196,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue)]
pub enum TimeUnit {
    Minute = 0,
    Hour = 1,
    Day = 2,
    Month = 3,
    Year = 4,
    Decade = 5,
    Normal = 6,
    Century = 7,
    #[description = "3 hours"]
    ThreeHours = 10,
    #[description = "6 hours"]
    SixHours = 11,
    #[description = "12 hours"]
    TwelveHours = 12,
    Seconds = 13,
}

impl TimeUnit {
    pub fn duration(&self, value: i64) -> Duration {
        match self {
            TimeUnit::Minute => Duration::minutes(value),
            TimeUnit::Hour => Duration::hours(value),
            TimeUnit::ThreeHours => Duration::hours(value * 3),
            TimeUnit::SixHours => Duration::hours(value * 6),
            TimeUnit::TwelveHours => Duration::hours(value * 12),
            TimeUnit::Day => Duration::hours(value * 24),
            TimeUnit::Month => Duration::hours(value * 730),
            TimeUnit::Year => Duration::hours(value * 8760),
            TimeUnit::Decade => Duration::hours(value * 87600),
            TimeUnit::Normal => Duration::hours(value * 262800),
            TimeUnit::Century => Duration::hours(value * 876000),
            TimeUnit::Seconds => Duration::seconds(value),
        }
    }
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue)]
pub enum TypeOfStatisticalProcessing {
    Average = 0,
    Accumulation = 1,
    Maximum = 2,
    Minimum = 3,
    #[description = "value at the end of the time range minus value at the beginning"]
    Difference = 4,
    RootMeanSquare = 5,
    StandardDeviation = 6,
    #[description = "temporal variance"]
    Covariance = 7,
    #[description = "value at the beginning of the time range minus value at the end"]
    DifferenceInv = 8,
    Ratio = 9,
    StandardizedAnomaly = 10,
    Summation = 11,
    ReturnPeriod = 12,
	Missing = 255,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue)]
pub enum TypeOfTimeInterval {
	Reserved = 0,
	#[description = "successive times processed have same forecast time, start time of forecast is incremented."]
	SameForecastIncrementedStartTime = 1,
	#[description = "successive times processed have same start time of forecast, forecast time is incremented"]
	SameStartTimeIncrementedForecastTime = 2,
	#[description = "Successive times processed have start time of forecast incremented and forecast time decremented so that valid time remains constant."]
	IncrementedStartTimeDecrementedForecastTime = 3,
	#[description = "Successive times processed have start time of forecast decremented and forecast time incremented so that valid time remains constant."]
	DecrementedStartTimeIncrementedForecastTime = 4,
	#[description = "Floating subinterval of time between forecast time and end of overall time interval"]
	FloatingSubintervfal = 5,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue)]
pub enum DerivedForecastType {
    UnweightedMean = 0,
    WeightedMean = 1,
    StandardDeviation = 2, 
    NormalizedStandardDeviation = 3, 
    Spread = 4,
    LargeAnomaly = 5,
    UnweightedMeanOfClustered = 6, 
    InterquartileRange = 7, 
    Minimum = 8, 
    Maximum = 9,
    UnweightedMode = 192, 
    TenthPercentile = 193, 
    FiftiethPercentile = 194, 
    NinetiethPercentile = 195, 
    StatisticallyWeighted = 196, 
    ClimatePercentile = 197, 
    DeviationOfMeanFromClimatology = 198, 
    ExtremeForecastIndex = 199, 
    EquallyWeightedMena = 200, 
    FifthPercentile = 201, 
    TwentyFifthPercentile = 202, 
    SeventyFifthPercentile = 203, 
    NinetyFifthPercentile = 204, 
    Missing = 255, 
}

pub trait ProductDiscipline {
    fn from_category_parameter(category: u8, parameter: u8) -> Self;

    fn parameter(&self) -> Option<Parameter>;
}

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
        19 => "physical atmospheric properties",
        _ => "other",
    }
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter)]
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

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter)]
pub enum WavesProduct {
    #[description = "primary wave spectra"]
    #[abbrev = "WVSP1"]
    #[unit = "-"]
    WaveSpectra1 = 0,
    #[description = "secondary wave spectra"]
    #[abbrev = "-"]
    #[unit = "WVSP2"]
    WaveSpectra2 = 1,
    #[description = "tertiary wave spectra"]
    #[abbrev = "-"]
    #[unit = "WVSP3"]
    WaveSpectra3 = 2,
    #[description = "significant wave height of combined wind and swell waves"]
    #[abbrev = "HTSGW"]
    #[unit = "m"]
    SignificantWaveHeight = 3,
    #[description = "direction of wind waves"]
    #[abbrev = "WVDIR"]
    #[unit = "degree"]
    WindWaveDirection = 4,
    #[description = "significant height of wind waves"]
    #[abbrev = "WVHGT"]
    #[unit = "m"]
    WindSignificantWaveHeight = 5,
    #[description = "mean period of wind waves"]
    #[abbrev = "WVPER"]
    #[unit = "s"]
    WindWaveMeanPeriod = 6,
    #[description = "direction of swell waves"]
    #[abbrev = "SWDIR"]
    #[unit = "degree"]
    SwellWaveDirection = 7,
    #[description = "significant height of swell waves"]
    #[abbrev = "SWELL"]
    #[unit = "m"]
    SwellSignificantWaveHeight = 8,
    #[description = "mean period of swell waves"]
    #[abbrev = "SWPER"]
    #[unit = "s"]
    SwellMeanPeriod = 9,
    #[description = "primary wave direction"]
    #[abbrev = "DIRPW"]
    #[unit = "degree"]
    PrimaryWaveDirection = 10,
    #[description = "primary wave mean period"]
    #[abbrev = "PERPW"]
    #[unit = "s"]
    PrimaryMeanPeriod = 11,
    #[description = "secondary wave direction"]
    #[abbrev = "DIRSW"]
    #[unit = "degree"]
    SecondaryWaveDirection = 12,
    #[description = "secondary wave mean period"]
    #[abbrev = "PERSW"]
    #[unit = "s"]
    SecondaryMeanPeriod = 13,
    #[description = "direction of combined wind and swell waves"]
    #[abbrev = "WWSDIR"]
    #[unit = "degree"]
    CombinedWaveDirection = 14,
    #[description = "mean period of combined wind and swell waves"]
    #[abbrev = "MWSPER"]
    #[unit = "s"]
    CombinedMeanPeriod = 15,
    #[description = "coefficient of drag with waves"]
    #[abbrev = "CDWW"]
    #[unit = "-"]
    WaveDragCoefficient = 16,
    #[description = "friction velocity"]
    #[abbrev = "FRICV"]
    #[unit = "ms-1"]
    FrictionVelocity = 17,
    #[description = "wave stress"]
    #[abbrev = "WSTR"]
    #[unit = "Nm-2"]
    WaveStress = 18,
    #[description = "normalized wave stress"]
    #[abbrev = "NWSTR"]
    #[unit = "-"]
    NormalizedWaveStress = 19,
    #[description = "mean square slope of waves"]
    #[abbrev = "MSSW"]
    #[unit = "-"]
    MeanSquareWaveSlope = 20,
    #[description = "u-component of stokes drift"]
    #[abbrev = "USSD"]
    #[unit = "ms-1"]
    UComponentStokesDrift = 21,
    #[description = "v-component of stokes drift"]
    #[abbrev = "VSSD"]
    #[unit = "ms-1"]
    VComponentStokesDrift = 22,
    #[description = "period of maximumm individual wave height"]
    #[abbrev = "PMAXWH"]
    #[unit = "s"]
    MaxWaveHeightPeriod = 23,
    #[description = "maximum individual wave height"]
    #[abbrev = "MAXWH"]
    #[unit = "m"]
    MaxWaveHeight = 24,
    #[description = "inverse mean wave frequency"]
    #[abbrev = "IMWF"]
    #[unit = "s"]
    InverseMeanWaveFrequency = 25,
    #[description = "inverse mean frequency of the wind waves"]
    #[abbrev = "IMFWW"]
    #[unit = "s"]
    InverseMeanWindWaveFrequency = 26,
    #[description = "inverse mean frequency of the total swell"]
    #[abbrev = "IMFTSW"]
    #[unit = "s"]
    InverseMeanTotalSwellFrequency = 27,
    #[description = "mean zero crossing wave period"]
    #[abbrev = "MZWPER"]
    #[unit = "s"]
    MeanZeroCrossingWavePeriod = 28,
    #[description = "mean zero crossing period of the wind waves"]
    #[abbrev = "MZPWW"]
    #[unit = "s"]
    MeanZeroCrossingWindWavePeriod = 29,
    #[description = "mean zero crossing of the total swell"]
    #[abbrev = "MZPTSW"]
    #[unit = "s"]
    MeanZeroCrossingTotalSwellPeriod = 30,
    #[description = "wave directional width"]
    #[abbrev = "WDIRW"]
    #[unit = "-"]
    DirectionalWidthWaves = 31,
    #[description = "directional width of the wind waves"]
    #[abbrev = "DIRWWW"]
    #[unit = "-"]
    DirectionalWidthWindWaves = 32,
    #[description = "directional width of the total swell"]
    #[abbrev = "DIRWTS"]
    #[unit = "-"]
    DirectionalWidthTotalSwell = 33,
    #[description = "peak wave period"]
    #[abbrev = "PWPER"]
    #[unit = "s"]
    PeakWavePeriod = 34,
    #[description = "peak period of the wind waves"]
    #[abbrev = "PPERWW"]
    #[unit = "s"]
    PeakWindWavePeriod = 35,
    #[description = "peak period of the total swell"]
    #[abbrev = "PPERTS"]
    #[unit = "s"]
    PeakTotalSwellPeriod = 36,
    #[description = "altimeter wave height"]
    #[abbrev = "ALTWH"]
    #[unit = "m"]
    AltimeterWaveHeight = 37,
    #[description = "altimeter corrected wave height"]
    #[abbrev = "ALCWH"]
    #[unit = "m"]
    AltimeterCorrectedWaveHeight = 38,
    #[description = "altimeter range relative correction"]
    #[abbrev = "ALRRC"]
    #[unit = "-"]
    AltimeterRangeRelativeCorrection = 39,
    #[description = "10 meter neutral wind speed over waves"]
    #[abbrev = "MNWSOW"]
    #[unit = "ms-1"]
    NeutralWindSpeedOverWaves = 40,
    #[description = "10 meter wind direction over waves"]
    #[abbrev = "MWDIRW"]
    #[unit = "degree"]
    WindDirectionOverWaves = 41,
    #[description = "wave energy spectrum"]
    #[abbrev = "WESP"]
    #[unit = "m-2 s rad-1"]
    WaveEnergySpectrum = 42,
    #[description = "kurtosis of the sea surface elevation due to waves"]
    #[abbrev = "KSSEW"]
    #[unit = "-"]
    KurtosisFromWaves = 43,
    #[description = "benjamin-feir index"]
    #[abbrev = "BENINX"]
    #[unit = "-"]
    BenjaminFeirIndex = 44,
    #[description = "spectral peakedness factor"]
    #[abbrev = "SPFTR"]
    #[unit = "s-1"]
    SpectralPeakednessFactor = 45,
    #[description = "wave steepness"]
    #[abbrev = "WSTP"]
    #[unit = "porportion"]
    WaveSteepness = 192,
    #[description = "wave length"]
    #[abbrev = "WLENG"]
    #[unit = "-"]
    WaveLength = 193,
    Missing = 255,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter)]
pub enum CurrentsProduct {
    #[description = "current direction"]
    #[abbrev = "DIRC"]
    #[unit = "degree True"]
    CurrentDirection = 0,
    #[description = "current speed"]
    #[abbrev = "SPC"]
    #[unit = "ms-1"]
    CurrentSpeed = 1,
    #[description = "u component of current speed"]
    #[abbrev = "UOGRD"]
    #[unit = "ms-1"]
    CurrentSpeedU = 2,
    #[description = "v component of current speed"]
    #[abbrev = "VOGRD"]
    #[unit = "ms-1"]
    CurrentSpeedV = 3,
    #[description = "rip current occurance probability"]
    #[abbrev = "RIPCOP"]
    #[unit = "%"]
    RipCurrentProbability = 4,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter)]
pub enum SurfacePropertiesProduct {
    #[description = "water temperature"]
    #[abbrev = "WTMP"]
    #[unit = "K"]
    WaterTemperature = 0,
    #[description = "deviation of sea level from mean"]
    #[abbrev = "DSLM"]
    #[unit = "m"]
    DeviationOfSeaLevelMean = 1,
    #[description = "total water level accounting for tide, wind and waves"]
    #[abbrev = "TWLWAV"]
    #[unit = "m"]
    TotalWaterLevelTideWindWaves = 205,
    #[description = "total water level increase due to waves"]
    #[abbrev = "RUNUP"]
    #[unit = "m"]
    TotalWaterLevelIncreaseWaves = 206,
    #[description = "mean water level increase due to waves"]
    #[abbrev = "SETUP"]
    #[unit = "m"]
    MeanWaterLevelIncreaseWaves = 207,
    #[description = "time varying water level increase due to waves"]
    #[abbrev = "SWASH"]
    #[unit = "m"]
    TimeVaryingWaterLevelIncreaseWaves = 208,
    #[description = "total water level above dune toe"]
    #[abbrev = "TWLDT"]
    #[unit = "m"]
    TotalWaterLevelAboveDuneToe = 209,
    #[description = "total water level above dune crest"]
    #[abbrev = "TWLDC"]
    #[unit = "m"]
    TotalWaterLevelAboveDuneCrest = 210,
    #[description = "erosion occurrence probability"]
    #[abbrev = "EROSNP"]
    #[unit = "%"]
    ErosionOccuranceProbability = 252,
    #[description = "overwash occurrence probability"]
    #[abbrev = "OWASHP"]
    #[unit = "%"]
    OverwashOccuranceProbability = 253,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter)]
pub enum IceProduct {
    #[description = "ice cover"]
    #[abbrev = "ICEC"]
    #[unit = "porportion"]
    IceCover = 0, 
    #[description = "ice thickness"]
    #[abbrev = "ICETK"]
    #[unit = "m"]
    IceThickness = 1,
    Missing = 255,
}

pub fn oceanographic_parameter(category: u8, parameter: u8) -> Option<Parameter> {
    match category {
        0 => Some(Parameter::from(WavesProduct::from(parameter))),
        1 => Some(Parameter::from(CurrentsProduct::from(parameter))),
        2 => Some(Parameter::from(IceProduct::from(parameter))),
        3 => Some(Parameter::from(SurfacePropertiesProduct::from(parameter))),
        _ => None,
    }
}

pub fn oceanographic_category(category: u8) -> &'static str {
    match category {
        0 => "waves",
        1 => "currents",
        2 => "ice",
        3 => "surface",
        4 => "subsurface",
        _ => "misc",
    }
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter)]
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
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter)]
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
