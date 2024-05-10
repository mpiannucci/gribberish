use chrono::{DateTime, Duration, Utc};
use gribberish_types::Parameter;

use super::{
    parameters::{category, parameter},
    tables::{
        DerivedForecastType, FixedSurfaceType, GeneratingProcess, TimeUnit,
        TypeOfStatisticalProcessing,
    },
};

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
        category(self.discipline(), self.category_value())
    }

    fn parameter(&self) -> Option<Parameter> {
        parameter(
            self.discipline(),
            self.category_value(),
            self.parameter_value(),
        )
    }

    fn time_interval_duration(&self) -> Duration {
        let time_unit = self.time_unit();
        time_unit.duration(self.time_interval() as i64)
    }

    fn time_increment_duration(&self) -> Option<Duration> {
        let time_unit = self.time_increment_unit()?;
        self.time_increment_interval()
            .map(|interval| time_unit.duration(interval as i64))
    }
}
