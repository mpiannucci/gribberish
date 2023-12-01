use gribberish_macros::{DisplayDescription, FromValue, ToParameter};
use gribberish_types::Parameter;

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
