use chrono::{DateTime, Utc};
use gribberish_types::Parameter;

use super::tables::{FixedSurfaceType, TimeUnit, GeneratingProcess};

pub trait ProductTemplate {
    fn category_value(&self) -> u8;
	fn parameter_value(&self) -> u8;
	fn category(&self) -> &'static str;
	fn parameter(&self) -> Option<Parameter>;
	fn generating_process(&self) -> GeneratingProcess;
	fn time_unit(&self) -> TimeUnit;
	fn forecast_datetime(&self, reference_date: DateTime<Utc>) -> DateTime<Utc>;
    fn first_fixed_surface_type(&self) -> FixedSurfaceType;
	fn first_fixed_surface_value(&self) -> Option<f64>;
    fn second_fixed_surface_type(&self) -> FixedSurfaceType;
	fn second_fixed_surface_value(&self) -> Option<f64>;
}