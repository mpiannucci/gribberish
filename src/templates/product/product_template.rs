use chrono::{DateTime, Utc, Duration};
use gribberish_types::Parameter;

use super::tables::{FixedSurfaceType, TimeUnit, GeneratingProcess, meteorological_category, land_surface_category, oceanographic_category, multiradar_category, meteorological_parameter, land_surface_parameter, oceanographic_parameter, multiradar_parameter, DerivedForecastType, TypeOfStatisticalProcessing};

pub trait ProductTemplate {
	fn discipline(&self) -> u8;
    fn category_value(&self) -> u8;
	fn parameter_value(&self) -> u8;
	fn generating_process(&self) -> GeneratingProcess;
	fn time_unit(&self) -> TimeUnit;
	fn time_increment_unit(&self) -> Option<TimeUnit>;
	fn time_interval(&self) -> u32;
	fn time_increment_interval(&self) -> Option<u32>;
	fn forecast_datetime(&self, reference_date: DateTime<Utc>) -> DateTime<Utc>;
	fn forecast_end_datetime(&self, reference_date: DateTime<Utc>) -> Option<DateTime<Utc>>;
    fn first_fixed_surface_type(&self) -> FixedSurfaceType;
	fn first_fixed_surface_value(&self) -> Option<f64>;
    fn second_fixed_surface_type(&self) -> FixedSurfaceType;
	fn second_fixed_surface_value(&self) -> Option<f64>;
	fn derived_forecast_type(&self) -> Option<DerivedForecastType>;
	fn statistical_process_type(&self) -> Option<TypeOfStatisticalProcessing>;

	fn category(&self) -> &'static str {
		let category = self.category_value();
		match self.discipline() {
			0 => meteorological_category(category),
			2 => land_surface_category(category),
			10 => oceanographic_category(category),
			209 => multiradar_category(category),
			_ => "",
		}
	}

	fn parameter(&self) -> Option<Parameter> {
		let category = self.category_value();
		let parameter = self.parameter_value();

		match self.discipline() {
			0 => meteorological_parameter(category, parameter),
			2 => land_surface_parameter(category, parameter),
			10 => oceanographic_parameter(category, parameter),
			209 => multiradar_parameter(category, parameter),
			_ => None,
		}
	}

	fn time_interval_duration(&self) -> Duration {
		let time_unit = self.time_unit();
		time_unit.duration(self.time_interval() as i64)
	}

	fn time_increment_duration(&self) -> Option<Duration> {
		let time_unit = self.time_increment_unit()?;
		self.time_increment_interval().map(|interval| time_unit.duration(interval as i64))
	}
}