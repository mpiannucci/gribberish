use chrono::{DateTime, Utc};
use gribberish_types::Parameter;

use super::tables::{FixedSurfaceType, TimeUnit, GeneratingProcess};

pub trait ProductTemplate {
    fn category_value(&self) -> u8;
	fn parameter_value(&self) -> u8;
	fn category(&self) -> &'static str;
	fn parameter(&self) -> Option<Parameter>;
	fn generating_process(&self) -> GeneratingProcess;
	fn observation_cutoff_hours_after_reference_time(&self) -> u16;
	fn observation_cutoff_minutes_after_cutoff_time(&self) -> u8;
	fn time_unit(&self) -> TimeUnit;
	fn forecast_datetime(&self, reference_date: DateTime<Utc>) -> DateTime<Utc>;
    fn first_fixed_surface_type(&self) -> FixedSurfaceType;
	fn first_fixed_surface_value(&self) -> Option<f64>;
    fn second_fixed_surface_type(&self) -> FixedSurfaceType;
	fn second_fixed_surface_value(&self) -> Option<f64>;
}