use crate::templates::template::{Template, TemplateType};
use chrono::{DateTime, Utc};

use super::product_template::{ProductTemplate, WavePeriodRange};
use super::tables::{EnsembleForecastType, FixedSurfaceType, GeneratingProcess, TimeUnit};
use super::WavePeriodRangeHorizontalForecastTemplate;

/// GRIB2 Product Definition Template 4.104
///
/// "Individual ensemble forecast, control and perturbed, at a horizontal level
/// or in a horizontal layer at a point in time for waves selected by period
/// range."
///
/// This is template 4.103 ([`WavePeriodRangeHorizontalForecastTemplate`]) with
/// the three template 4.1 ensemble octets (type of ensemble forecast,
/// perturbation number, number of forecasts in ensemble) appended after the
/// second fixed surface. Octets 10-45 share the 4.103 layout, so those fields
/// are read through a wrapped 4.103 template.
///
/// Used by the ECMWF IFS wave model for the period-banded significant wave
/// heights (`h1012`, `h1214`, ...) in both HRES (perturbation number 0) and ENS.
pub struct WavePeriodRangeEnsembleForecastTemplate {
    inner: WavePeriodRangeHorizontalForecastTemplate,
}

impl Template for WavePeriodRangeEnsembleForecastTemplate {
    fn data(&self) -> &[u8] {
        self.inner.data()
    }

    fn template_number(&self) -> u16 {
        104
    }

    fn template_type(&self) -> TemplateType {
        TemplateType::Product
    }

    fn template_name(&self) -> &str {
        "Individual ensemble forecast, control and perturbed, at a horizontal level or in a horizontal layer at a point in time for waves selected by period range"
    }
}

impl WavePeriodRangeEnsembleForecastTemplate {
    pub fn new(data: Vec<u8>, discipline: u8) -> Self {
        WavePeriodRangeEnsembleForecastTemplate {
            inner: WavePeriodRangeHorizontalForecastTemplate::new(data, discipline),
        }
    }

    /// Period range fields shared with template 4.103.
    pub fn wave_period_range_template(&self) -> &WavePeriodRangeHorizontalForecastTemplate {
        &self.inner
    }

    pub fn type_of_ensemble_forecast(&self) -> EnsembleForecastType {
        self.data()[45].into()
    }

    pub fn perturbation_number(&self) -> u8 {
        self.data()[46]
    }

    pub fn number_of_forecasts_in_ensemble(&self) -> u8 {
        self.data()[47]
    }
}

impl ProductTemplate for WavePeriodRangeEnsembleForecastTemplate {
    fn discipline(&self) -> u8 {
        self.inner.discipline()
    }

    fn category_value(&self) -> u8 {
        self.inner.category_value()
    }

    fn parameter_value(&self) -> u8 {
        self.inner.parameter_value()
    }

    fn generating_process(&self) -> GeneratingProcess {
        self.inner.generating_process()
    }

    fn time_unit(&self) -> TimeUnit {
        self.inner.time_unit()
    }

    fn time_increment_unit(&self) -> Option<TimeUnit> {
        None
    }

    fn time_interval(&self) -> i32 {
        self.inner.time_interval()
    }

    fn time_increment_interval(&self) -> Option<u32> {
        None
    }

    fn forecast_end_datetime(&self, _reference_date: DateTime<Utc>) -> Option<DateTime<Utc>> {
        None
    }

    fn first_fixed_surface_type(&self) -> FixedSurfaceType {
        self.inner.first_fixed_surface_type()
    }

    fn first_fixed_surface_value(&self) -> Option<f64> {
        self.inner.first_fixed_surface_value()
    }

    fn second_fixed_surface_type(&self) -> FixedSurfaceType {
        self.inner.second_fixed_surface_type()
    }

    fn second_fixed_surface_value(&self) -> Option<f64> {
        self.inner.second_fixed_surface_value()
    }

    fn derived_forecast_type(&self) -> Option<super::tables::DerivedForecastType> {
        None
    }

    fn statistical_process_type(&self) -> Option<super::tables::TypeOfStatisticalProcessing> {
        None
    }

    fn perturbation_number(&self) -> Option<u8> {
        Some(self.perturbation_number())
    }

    fn number_of_ensemble_members(&self) -> Option<u8> {
        Some(self.number_of_forecasts_in_ensemble())
    }

    fn wave_period_range(&self) -> Option<WavePeriodRange> {
        self.inner.wave_period_range()
    }
}
