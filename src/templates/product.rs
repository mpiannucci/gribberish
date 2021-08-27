use gribberish_macros::{DisplayDescription, FromValue, ToParameter};
use gribberish_types::Parameter;
use super::template::{Template, TemplateType};
use crate::utils::{read_u16_from_bytes, read_u32_from_bytes};
use chrono::{Utc, DateTime, Duration};

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue)]
pub enum ClusteringMethod {
	#[description = "anomoly correlation"]
	AnomolyCorrelation = 0,
	#[description = "root mean square"]
	RMS = 1,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue)]
pub enum FixedSurfaceTypes {
	#[description = "ground or water surface"]
	GroundOrWater = 1,
	#[description = "cloud base level"]
	CloudBase = 2,
	#[description = "cloud tops level"]
	CloudTop = 3, 
	#[description = "Ordered Sequence of Data"]
	OrderedSequence = 241,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue)]
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
	#[description = "depoint temperature"]
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
	PrecipitationRate = 5,
	#[description = "total precipitation"]
	#[abbrev = "APCP"]
	#[unit = "kgm-2"]
	TotalPrecipitation = 8,
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
}

fn meteorological_parameter(category: u8, parameter: u8) -> Option<Parameter> {
	match category {
		0 => Some(Parameter::from(TemperatureProduct::from(parameter))),
		1 => Some(Parameter::from(MoistureProduct::from(parameter))),
		2 => Some(Parameter::from(MomentumProduct::from(parameter))),
		3 => Some(Parameter::from(MassProduct::from(parameter))),
		_ => None,
	}
}

fn meteorological_category(category: u8) -> &'static str {
	match category {
		0 => "temperature",
		1 => "moisture",
		2 => "momentum",
		3 => "mass",
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
}

fn oceanographic_parameter(category: u8, parameter: u8) -> Option<Parameter> {
	match category {
		0 => Some(Parameter::from(WavesProduct::from(parameter))),
		_ => None,
	}
}

fn oceanographic_category(category: u8) -> &'static str {
	match category {
		0 => "waves",
		1 => "currents",
		2 => "ice",
		3 => "surface",
		4 => "subsurface",
		_ => "misc",
	}
}

pub enum ProductTemplate<'a> {
	HorizontalAnalysisForecast(HorizontalAnalysisForecastTemplate<'a>),
	Other,
}

impl <'a> ProductTemplate<'a> {
	pub fn from_template_number(template_number: u16, data: &'a[u8], discipline: u8) -> ProductTemplate {
		match template_number {
			0 => ProductTemplate::HorizontalAnalysisForecast(HorizontalAnalysisForecastTemplate{data, discipline}),
			_ => ProductTemplate::Other,
		}
	}
}

pub struct HorizontalAnalysisForecastTemplate<'a> {
	data: &'a[u8],
	discipline: u8,
}

impl <'a> Template for HorizontalAnalysisForecastTemplate<'a> {
	fn data(&self) -> &[u8] {
    	self.data
 	}

 	fn template_number(&self) -> u16 {
 	    0
 	}

 	fn template_type(&self) -> TemplateType {
 	    TemplateType::Product
 	}
 	
    fn template_name(&self) -> &str {
        "Analysis or forecast at a horizontal level or in a horizontal layer at a point in time"
    }
}

impl <'a> HorizontalAnalysisForecastTemplate<'a> {

	pub fn category_value(&self) -> u8 {
		self.data[9]
	}

	pub fn parameter_value(&self) -> u8{
		self.data[10]
	}

	pub fn category(&self) -> &'static str {
		let category = self.category_value();
		match self.discipline {
			0 => meteorological_category(category),
			10 => oceanographic_category(category),
			_ => "",
		}
	}

	pub fn parameter(&self) -> Option<Parameter> {
		let category = self.category_value();
		let parameter = self.parameter_value();

		match self.discipline {
			0 => meteorological_parameter(category, parameter),
			10 => oceanographic_parameter(category, parameter),
			_ => None,
		}
	}

	pub fn generating_process(&self) -> GeneratingProcess {
		self.data[12].into()
	}

	pub fn observation_cutoff_hours_after_reference_time(&self) -> u16 {
		read_u16_from_bytes(self.data, 14).unwrap_or(0)
	}

	pub fn observation_cutoff_minutes_after_cutoff_time(&self) -> u8 {
		self.data[16]
	}

	pub fn time_unit(&self) -> TimeUnit {
		self.data[17].into()
	}

	pub fn forecast_time(&self) -> u32 {
		read_u32_from_bytes(self.data, 18).unwrap_or(0)
	}

	pub fn forecast_datetime(&self, reference_date: DateTime<Utc>) -> DateTime<Utc> {
		let forecast_offset = self.forecast_time();
		let offset_duration: Duration = self.time_unit().duration(forecast_offset as i64);
		reference_date + offset_duration
	}

    pub fn first_fixed_surface_type(&self) -> FixedSurfaceTypes {
        self.data[22].into()
    }

    pub fn first_fixed_surface_scale_factor(&self) -> u8 {
        self.data[23]
    }

    pub fn first_fixed_surface_scaled_value(&self) -> u32 {
        read_u32_from_bytes(self.data, 24).unwrap_or(0)
    }

	pub fn array_index(&self) -> Option<usize> {
		match self.first_fixed_surface_type() {
			FixedSurfaceTypes::OrderedSequence => Some(self.first_fixed_surface_scaled_value() as usize), 
			_ => None,
		}
	}

    pub fn second_fixed_surface_type(&self) -> FixedSurfaceTypes {
        self.data[28].into()
    }

    pub fn second_fixed_surface_scale_factor(&self) -> u8 {
        self.data[29]
    }

    pub fn second_fixed_surface_scaled_value(&self) -> u32 {
        read_u32_from_bytes(self.data, 30).unwrap_or(0)
    }
}