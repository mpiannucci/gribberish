use grib_data_derive::{DisplayDescription, FromValue, Parameter};
use super::template::{Template, TemplateType};

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
	ThreeHours = 8,
	#[description = "6 hours"]
	SixHours = 9,
	#[description = "12 hours"]
	TwelveHours = 10,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, Parameter)]
pub enum TemperatureProduct {
	Temperature = 0,
	VirtualTemperature = 1,
	PotentialTemperature = 2,
	PseudoAdiabaticPotentialTemperature = 3,
	MaximumTemperature = 4,
	MinimumTemperature = 5,
	DewpointTemperature = 6,
	DewpointDepression = 7,
	LapseRate = 8,
	HeatIndex = 12, 
	WindChillFactor = 13,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, Parameter)]
pub enum MoistureProduct {
	SpecificHumidity = 0,
	RelativeHUmidity = 1,
	HumidityMixingRatio = 2,
	PrecipitableWater = 3, 
	Evaporation = 4,
	PrecipitationRate = 5,
	TotalPrecipitation = 8,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, Parameter)]
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
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, Parameter)]
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
