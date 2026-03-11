use crate::templates::template::{Template, TemplateType};
use crate::utils::{read_u16_from_bytes, read_u32_from_bytes};
use chrono::{prelude::*, Duration};

use super::product_template::ProductTemplate;
use super::tables::{
    FixedSurfaceType, GeneratingProcess, TimeUnit, TypeOfStatisticalProcessing,
};
use super::HorizontalAnalysisForecastTemplate;

/// Product Definition Template 4.9
/// Probability forecasts at a horizontal level or in a horizontal layer
/// in a continuous or non-continuous time interval
pub struct ProbabilityHorizontalTimeIntervalTemplate {
    data: Vec<u8>,
    discipline: u8,
}

impl Template for ProbabilityHorizontalTimeIntervalTemplate {
    fn data(&self) -> &[u8] {
        &self.data
    }

    fn template_number(&self) -> u16 {
        9
    }

    fn template_type(&self) -> TemplateType {
        TemplateType::Product
    }

    fn template_name(&self) -> &str {
        "Probability forecasts at a horizontal level or in a horizontal layer in a continuous or non-continuous time interval"
    }
}

impl ProbabilityHorizontalTimeIntervalTemplate {
    pub fn new(data: Vec<u8>, discipline: u8) -> Self {
        Self { data, discipline }
    }

    pub fn first_fixed_surface_scale_factor(&self) -> i8 {
        as_signed!(self.data[23], 8, i8)
    }

    pub fn first_fixed_surface_scaled_value(&self) -> i32 {
        as_signed!(read_u32_from_bytes(&self.data, 24).unwrap_or(0), 32, i32)
    }

    pub fn second_fixed_surface_scale_factor(&self) -> i8 {
        as_signed!(self.data[29], 8, i8)
    }

    pub fn second_fixed_surface_scaled_value(&self) -> i32 {
        as_signed!(read_u32_from_bytes(&self.data, 30).unwrap_or(0), 32, i32)
    }

    // Probability fields (octets 35-48, same as template 5 but shifted by probability fields)
    // Octet 35: forecast probability number
    // Octet 36: total number of forecast probabilities
    // Octet 37: probability type
    // Octets 38-41: lower limit scale factor + scaled value
    // Octets 42-47: upper limit scale factor + scaled value

    // Time interval fields start at octet 48
    pub fn valid_end_date(&self) -> DateTime<Utc> {
        let data = self.data();
        let year = read_u16_from_bytes(data, 47).unwrap_or(0) as i32;
        let month = data.get(49).copied().unwrap_or(0) as u32;
        let day = data.get(50).copied().unwrap_or(0) as u32;
        let hour = data.get(51).copied().unwrap_or(0) as u32;
        let minute = data.get(52).copied().unwrap_or(0) as u32;
        let second = data.get(53).copied().unwrap_or(0) as u32;

        Utc.with_ymd_and_hms(year, month, day, hour, minute, second)
            .unwrap()
    }

    pub fn number_of_time_ranges(&self) -> u8 {
        self.data().get(54).copied().unwrap_or(0)
    }

    pub fn number_of_values_missing_from_stats(&self) -> u32 {
        read_u32_from_bytes(self.data(), 55).unwrap_or(0)
    }

    pub fn statistical_process(&self) -> TypeOfStatisticalProcessing {
        self.data().get(59).copied().unwrap_or(255).into()
    }
}

impl ProductTemplate for ProbabilityHorizontalTimeIntervalTemplate {
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
        self.data[11].into()
    }

    fn time_unit(&self) -> TimeUnit {
        self.data[17].into()
    }

    fn time_increment_unit(&self) -> Option<TimeUnit> {
        self.data().get(66).map(|&v| v.into())
    }

    fn time_interval(&self) -> u32 {
        read_u32_from_bytes(&self.data, 18).unwrap_or(0)
    }

    fn time_increment_interval(&self) -> Option<u32> {
        if self.data().len() > 67 {
            Some(read_u32_from_bytes(self.data(), 67).unwrap_or(0))
        } else {
            None
        }
    }

    fn forecast_datetime(&self, reference_date: DateTime<Utc>) -> DateTime<Utc> {
        let offset_duration: Duration = self.time_interval_duration();
        reference_date + offset_duration
    }

    fn forecast_end_datetime(&self, _reference_date: DateTime<Utc>) -> Option<DateTime<Utc>> {
        Some(self.valid_end_date())
    }

    fn first_fixed_surface_type(&self) -> FixedSurfaceType {
        self.data[22].into()
    }

    fn first_fixed_surface_value(&self) -> Option<f64> {
        HorizontalAnalysisForecastTemplate::scale_value(
            self.first_fixed_surface_scale_factor(),
            self.first_fixed_surface_scaled_value(),
        )
    }

    fn second_fixed_surface_type(&self) -> FixedSurfaceType {
        self.data[28].into()
    }

    fn second_fixed_surface_value(&self) -> Option<f64> {
        HorizontalAnalysisForecastTemplate::scale_value(
            self.second_fixed_surface_scale_factor(),
            self.second_fixed_surface_scaled_value(),
        )
    }

    fn derived_forecast_type(&self) -> Option<super::tables::DerivedForecastType> {
        None
    }

    fn statistical_process_type(&self) -> Option<TypeOfStatisticalProcessing> {
        Some(self.statistical_process())
    }
}
