use chrono::Duration;
use gribberish_macros::{DisplayDescription, FromValue, ToParameter};
use gribberish_types::Parameter;

use crate::error::GribberishError;

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
    #[name = "surface"]
    #[description = "ground or water surface"]
    GroundOrWater = 1,
    #[name = "cloud base"]
    #[description = "cloud base level"]
    CloudBase = 2,
    #[name = "cloud top"]
    #[description = "cloud tops level"]
    CloudTop = 3,
    #[description = "level of 0oc isotherm"]
    ZeroDegreeIsotherm = 4,
    #[description = "level of adiabatic condensation lifted from the surface"]
    AdiabaticCondensationLifted = 5,
    #[description = "maximum wind level"]
    MaximumWindLevel = 6,
    #[description = "tropopause"]
    Tropopause = 7,
    #[description = "sea bottom"]
    SeaBottom = 9,
    #[name = "top of atmosphere"]
    #[description = "nominal top of the atmosphere"]
    NominalTopOfAtmosphere = 8,
    #[name = "entire atmosphere"]
    #[description = "entire atmosphere"]
    EntireAtmosphere = 10,
    #[description = "level of free convection"]
    LevelOfFreeConvection = 14,
    #[description = "isothermal level"]
    IsothermalLevel = 20,
    #[name = "mb"]
    #[description = "isobaric surface"]
    IsobaricSurface = 100,
    #[description = "mean sea level"]
    MeanSeaLevel = 101,
    #[description = "specific altitude above mean sea level"]
    SpecificAltitudeAboveMeanSeaLevel = 102,
    #[name = "above ground"]
    #[unit = "m"]
    #[description = "specific height level above ground"]
    SpecifiedHeightLevelAboveGround = 103,
    #[description = "sigma level"]
    SigmaLevel = 104,
    #[description = "hybrid level"]
    HybridLevel = 105,
    #[description = "depth below land surface"]
    DepthBelowLandSurface = 106,
    #[description = "level at specified pressure difference from ground to level"]
    LevelAtSpecifiedPressureDifferenceFromGroundToLevel = 108,
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
    #[description = "highest tropospheric freezing level"]
    HighestTroposphericFreezingLevel = 204,
    #[description = "boundary layer cloud layer"]
    BoundaryLayerCloudLayer = 211,
    #[description = "low cloud layer"]
    LowCloudLayer = 214,
    #[name = "cloud ceiling"]
    #[description = "cloud ceiling"]
    CloudCeiling = 215,
    #[name = "planetary boundary layer"]
    #[description = "planetary boundary layer"]
    PlanetaryBoundaryLayer = 220,
    #[description = "middle cloud layer"]
    MiddleCloudLayer = 224,
    #[description = "high cloud layer"]
    HighCloudLayer = 234,
    #[name = "sequence"]
    #[description = "ordered Sequence of Data"]
    OrderedSequence = 241,
    #[description = "equilibrium level"]
    EquilibriumLevel = 247,
    #[description = "missing"]
    Missing = 255,
}

impl FixedSurfaceType {
    pub fn is_single_level(&self) -> bool {
        !self.is_vertical_level() && !self.is_sequence_level()
    }

    pub fn is_sequence_level(&self) -> bool {
        match self {
            FixedSurfaceType::OrderedSequence => true,
            _ => false,
        }
    }

    pub fn is_vertical_level(&self) -> bool {
        match self {
            FixedSurfaceType::SigmaLevel => true,
            FixedSurfaceType::HybridLevel => true,
            FixedSurfaceType::EtaLevel => true,
            FixedSurfaceType::SnowLevel => true,
            FixedSurfaceType::SigmaHeightLevel => true,
            FixedSurfaceType::GeneralizedVerticalHeightCoordinate => true,
            FixedSurfaceType::DepthBelowSeaLevel => true,
            FixedSurfaceType::DepthBelowWaterSurface => true,
            FixedSurfaceType::MixingLayer => true,
            FixedSurfaceType::IsobaricSurface => true,
            FixedSurfaceType::LevelAtSpecifiedPressureDifferenceFromGroundToLevel => true,
            _ => false,
        }
    }

    pub fn coordinate_name(&self) -> &'static str {
        match self {
            FixedSurfaceType::GroundOrWater => "sfc",
            FixedSurfaceType::CloudBase => "clb",
            FixedSurfaceType::CloudTop => "clt",
            FixedSurfaceType::MaximumWindLevel => "mwl",
            FixedSurfaceType::Tropopause => "tro",
            FixedSurfaceType::SeaBottom => "bot",
            FixedSurfaceType::EntireAtmosphere => "atm",
            FixedSurfaceType::IsothermalLevel => "isotherm",
            FixedSurfaceType::MeanSeaLevel => "msl",
            FixedSurfaceType::SpecificAltitudeAboveMeanSeaLevel => "asl",
            FixedSurfaceType::SpecifiedHeightLevelAboveGround => "hag",
            FixedSurfaceType::SigmaLevel => "sigma",
            FixedSurfaceType::HybridLevel => "hybid",
            FixedSurfaceType::EtaLevel => "eta",
            FixedSurfaceType::SnowLevel => "snow",
            FixedSurfaceType::SigmaHeightLevel => "sigma_h",
            FixedSurfaceType::GeneralizedVerticalHeightCoordinate => "height",
            FixedSurfaceType::DepthBelowSeaLevel => "depth_bsl",
            FixedSurfaceType::DepthBelowWaterSurface => "depth_bws",
            FixedSurfaceType::MixingLayer => "mixing",
            FixedSurfaceType::EntireAtmosphereAsSingleLayer => "entire_atm",
            FixedSurfaceType::EntireOceanAsSingleLayer => "entire_ocean",
            FixedSurfaceType::OrderedSequence => "seq",
            FixedSurfaceType::Missing => "",
            FixedSurfaceType::ZeroDegreeIsotherm => "zero_deg_isotherm",
            FixedSurfaceType::AdiabaticCondensationLifted => "adiabatic_condensation_lifted",
            FixedSurfaceType::NominalTopOfAtmosphere => "nominal_top",
            FixedSurfaceType::LevelOfFreeConvection => "lfc",
            FixedSurfaceType::DepthBelowLandSurface => "depth_bls",
            FixedSurfaceType::HighestTroposphericFreezingLevel => "htfl",
            FixedSurfaceType::BoundaryLayerCloudLayer => "bndry_cloud",
            FixedSurfaceType::LowCloudLayer => "lcl",
            FixedSurfaceType::CloudCeiling => "cld_ceiling",
            FixedSurfaceType::MiddleCloudLayer => "mcl",
            FixedSurfaceType::HighCloudLayer => "hcl",
            FixedSurfaceType::EquilibriumLevel => "eqm",
            FixedSurfaceType::IsobaricSurface => "isobar",
            FixedSurfaceType::LevelAtSpecifiedPressureDifferenceFromGroundToLevel => "pres_diff",
            FixedSurfaceType::PlanetaryBoundaryLayer => "pbl",
        }
    }
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

impl GeneratingProcess {
    pub fn abbv(&self) -> String {
        match self {
            GeneratingProcess::Analysis => "anl".to_string(),
            GeneratingProcess::Initialization => "ini".to_string(),
            GeneratingProcess::Forecast => "fcst".to_string(),
            GeneratingProcess::BiasCorrectedForecast => "bc".to_string(),
            GeneratingProcess::EnsembleForecast => "ens".to_string(),
            GeneratingProcess::ProbabilityForecast => "prob".to_string(),
            GeneratingProcess::ForecastError => "err".to_string(),
            GeneratingProcess::AnalysisError => "anl_err".to_string(),
            GeneratingProcess::Observation => "obs".to_string(),
            GeneratingProcess::Climatological => "clim".to_string(),
            GeneratingProcess::ProbabilityWeightedForecast => "pwt".to_string(),
            GeneratingProcess::BiasCorrectedEnsembleForecast => "bc_ens".to_string(),
            GeneratingProcess::PostProcessedAnalysis => "pp_anl".to_string(),
            GeneratingProcess::PostProcessedForecast => "pp_fcst".to_string(),
            GeneratingProcess::Nowcast => "now".to_string(),
            GeneratingProcess::Hindcast => "hind".to_string(),
            GeneratingProcess::PhysicalRetrieval => "phy".to_string(),
            GeneratingProcess::RegressionAnalysis => "reg".to_string(),
            GeneratingProcess::DifferenceBetweenTwoForecasts => "diff".to_string(),
            GeneratingProcess::ForecastConfidenceIndicator => "fci".to_string(),
            GeneratingProcess::ProbabilityMatchedMean => "pmm".to_string(),
            GeneratingProcess::NeighborhoodProbability => "nprob".to_string(),
            GeneratingProcess::BiasCorrectedDownscaledEnsembleForecast => "bc_dens".to_string(),
            GeneratingProcess::PerturbedAnalysisForEnsembleInitialization => "pert_anl".to_string(),
        }
    }
}

#[repr(u8)]
#[derive(Clone, Eq, PartialEq, Debug, DisplayDescription, FromValue)]
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

impl TryFrom<&str> for TimeUnit {
    type Error = GribberishError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "minute" => Ok(TimeUnit::Minute),
            "hour" => Ok(TimeUnit::Hour),
            "day" => Ok(TimeUnit::Day),
            "month" => Ok(TimeUnit::Month),
            "year" => Ok(TimeUnit::Year),
            "decade" => Ok(TimeUnit::Decade),
            "normal" => Ok(TimeUnit::Normal),
            "century" => Ok(TimeUnit::Century),
            "3 hours" => Ok(TimeUnit::ThreeHours),
            "6 hours" => Ok(TimeUnit::SixHours),
            "12 hours" => Ok(TimeUnit::TwelveHours),
            "seconds" => Ok(TimeUnit::Seconds),
            _ => Err(GribberishError::TimeUnitError(value.to_string())),
        }
    }
}

#[repr(u8)]
#[derive(Clone, Eq, PartialEq, Debug, DisplayDescription, FromValue)]
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

impl TypeOfStatisticalProcessing {
    pub fn abbv(&self) -> String {
        match self {
            TypeOfStatisticalProcessing::Average => "avg".to_string(),
            TypeOfStatisticalProcessing::Accumulation => "acc".to_string(),
            TypeOfStatisticalProcessing::Maximum => "max".to_string(),
            TypeOfStatisticalProcessing::Minimum => "min".to_string(),
            TypeOfStatisticalProcessing::Difference => "diff".to_string(),
            TypeOfStatisticalProcessing::RootMeanSquare => "rms".to_string(),
            TypeOfStatisticalProcessing::StandardDeviation => "std".to_string(),
            TypeOfStatisticalProcessing::Covariance => "cov".to_string(),
            TypeOfStatisticalProcessing::DifferenceInv => "diff_inv".to_string(),
            TypeOfStatisticalProcessing::Ratio => "ratio".to_string(),
            TypeOfStatisticalProcessing::StandardizedAnomaly => "std_anom".to_string(),
            TypeOfStatisticalProcessing::Summation => "sum".to_string(),
            TypeOfStatisticalProcessing::ReturnPeriod => "return_period".to_string(),
            TypeOfStatisticalProcessing::Missing => "".to_string(),
        }
    }
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
