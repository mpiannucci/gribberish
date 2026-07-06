use crate::templates::template::{Template, TemplateType};
use crate::utils::read_u32_from_bytes;
use chrono::{DateTime, Utc};

use super::product_template::{ProductTemplate, WavePeriodRange};
use super::tables::{FixedSurfaceType, GeneratingProcess, TimeUnit};

/// GRIB2 Product Definition Template 4.103
///
/// "Analysis or forecast at a horizontal level or in a horizontal layer at a
/// point in time for waves selected by period range."
///
/// This is identical to template 4.0 ([`super::HorizontalAnalysisForecastTemplate`])
/// with an 11 octet "wave period range" block inserted immediately after the
/// parameter number. Everything from the generating process onwards is therefore
/// shifted by 11 octets relative to template 4.0.
///
/// Used by the ECMWF AIFS wave model for the period-banded significant wave height
/// parameters (e.g. `h1012`, the significant wave height of all waves with periods
/// in the inclusive range from 10 to 12 seconds).
pub struct WavePeriodRangeHorizontalForecastTemplate {
    data: Vec<u8>,
    discipline: u8,
}

impl Template for WavePeriodRangeHorizontalForecastTemplate {
    fn data(&self) -> &[u8] {
        &self.data
    }

    fn template_number(&self) -> u16 {
        103
    }

    fn template_type(&self) -> TemplateType {
        TemplateType::Product
    }

    fn template_name(&self) -> &str {
        "Analysis or forecast at a horizontal level or in a horizontal layer at a point in time for waves selected by period range"
    }
}

impl WavePeriodRangeHorizontalForecastTemplate {
    pub fn new(data: Vec<u8>, discipline: u8) -> Self {
        WavePeriodRangeHorizontalForecastTemplate { data, discipline }
    }

    /// Type of wave period interval (GRIB2 code table 4.91). A value of `7`
    /// means "between first and second limit, inclusive of both limits".
    pub fn wave_period_interval_type(&self) -> u8 {
        self.data[11]
    }

    pub fn lower_wave_period_scale_factor(&self) -> i8 {
        as_signed!(self.data[12], 8, i8)
    }

    pub fn lower_wave_period_scaled_value(&self) -> i32 {
        as_signed!(read_u32_from_bytes(&self.data, 13).unwrap_or(0), 32, i32)
    }

    pub fn upper_wave_period_scale_factor(&self) -> i8 {
        as_signed!(self.data[17], 8, i8)
    }

    pub fn upper_wave_period_scaled_value(&self) -> i32 {
        as_signed!(read_u32_from_bytes(&self.data, 18).unwrap_or(0), 32, i32)
    }

    /// Lower limit of the wave period range, in seconds.
    pub fn lower_wave_period(&self) -> Option<f64> {
        WavePeriodRangeHorizontalForecastTemplate::scale_value(
            self.lower_wave_period_scale_factor(),
            self.lower_wave_period_scaled_value(),
        )
    }

    /// Upper limit of the wave period range, in seconds.
    pub fn upper_wave_period(&self) -> Option<f64> {
        WavePeriodRangeHorizontalForecastTemplate::scale_value(
            self.upper_wave_period_scale_factor(),
            self.upper_wave_period_scaled_value(),
        )
    }

    pub fn first_fixed_surface_scale_factor(&self) -> i8 {
        as_signed!(self.data[34], 8, i8)
    }

    pub fn first_fixed_surface_scaled_value(&self) -> i32 {
        as_signed!(read_u32_from_bytes(&self.data, 35).unwrap_or(0), 32, i32)
    }

    pub fn second_fixed_surface_scale_factor(&self) -> i8 {
        as_signed!(self.data[40], 8, i8)
    }

    pub fn second_fixed_surface_scaled_value(&self) -> i32 {
        as_signed!(read_u32_from_bytes(&self.data, 41).unwrap_or(0), 32, i32)
    }

    pub fn scale_value(factor: i8, scaled_value: i32) -> Option<f64> {
        let factor = if factor == i8::MIN + 1 {
            0
        } else {
            factor as i32
        };
        let scale_factor = 10_f64.powi(-factor);

        if scaled_value == i32::MIN + 1 {
            None
        } else {
            Some(scaled_value as f64 * scale_factor)
        }
    }
}

impl ProductTemplate for WavePeriodRangeHorizontalForecastTemplate {
    fn discipline(&self) -> u8 {
        self.discipline
    }

    fn category_value(&self) -> u8 {
        self.data[9]
    }

    fn parameter_value(&self) -> u8 {
        self.data[10]
    }

    fn generating_process(&self) -> GeneratingProcess {
        self.data[22].into()
    }

    fn time_unit(&self) -> TimeUnit {
        self.data[28].into()
    }

    fn time_increment_unit(&self) -> Option<TimeUnit> {
        None
    }

    fn time_interval(&self) -> u32 {
        read_u32_from_bytes(&self.data, 29).unwrap_or(0)
    }

    fn time_increment_interval(&self) -> Option<u32> {
        None
    }

    fn forecast_datetime(&self, reference_date: DateTime<Utc>) -> DateTime<Utc> {
        let offset_duration = self.time_interval_duration();
        reference_date + offset_duration
    }

    fn forecast_end_datetime(&self, _reference_date: DateTime<Utc>) -> Option<DateTime<Utc>> {
        None
    }

    fn first_fixed_surface_type(&self) -> FixedSurfaceType {
        self.data[33].into()
    }

    fn first_fixed_surface_value(&self) -> Option<f64> {
        WavePeriodRangeHorizontalForecastTemplate::scale_value(
            self.first_fixed_surface_scale_factor(),
            self.first_fixed_surface_scaled_value(),
        )
    }

    fn second_fixed_surface_type(&self) -> FixedSurfaceType {
        self.data[39].into()
    }

    fn second_fixed_surface_value(&self) -> Option<f64> {
        WavePeriodRangeHorizontalForecastTemplate::scale_value(
            self.second_fixed_surface_scale_factor(),
            self.second_fixed_surface_scaled_value(),
        )
    }

    fn derived_forecast_type(&self) -> Option<super::tables::DerivedForecastType> {
        None
    }

    fn statistical_process_type(&self) -> Option<super::tables::TypeOfStatisticalProcessing> {
        None
    }

    fn wave_period_range(&self) -> Option<WavePeriodRange> {
        Some((self.lower_wave_period(), self.upper_wave_period()))
    }
}
