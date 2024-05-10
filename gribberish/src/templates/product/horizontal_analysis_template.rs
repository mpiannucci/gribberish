use crate::templates::template::{Template, TemplateType};
use crate::utils::{read_u16_from_bytes, read_u32_from_bytes};
use chrono::{DateTime, Utc};

use super::product_template::ProductTemplate;
use super::tables::{FixedSurfaceType, GeneratingProcess, TimeUnit};

pub struct HorizontalAnalysisForecastTemplate {
    data: Vec<u8>,
    discipline: u8,
}

impl Template for HorizontalAnalysisForecastTemplate {
    fn data(&self) -> &[u8] {
        &self.data
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

impl HorizontalAnalysisForecastTemplate {
    pub fn new(data: Vec<u8>, discipline: u8) -> Self {
        HorizontalAnalysisForecastTemplate { data, discipline }
    }

    pub fn category_value(&self) -> u8 {
        self.data[9]
    }

    pub fn parameter_value(&self) -> u8 {
        self.data[10]
    }

    pub fn generating_process(&self) -> GeneratingProcess {
        self.data[11].into()
    }

    pub fn observation_cutoff_hours_after_reference_time(&self) -> u16 {
        read_u16_from_bytes(&self.data, 14).unwrap_or(0)
    }

    pub fn observation_cutoff_minutes_after_cutoff_time(&self) -> u8 {
        self.data[16]
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

    pub fn array_index(&self) -> Option<usize> {
        match self.first_fixed_surface_type() {
            FixedSurfaceType::OrderedSequence => {
                Some(self.first_fixed_surface_scaled_value() as usize)
            }
            _ => None,
        }
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

impl ProductTemplate for HorizontalAnalysisForecastTemplate {
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
        None
    }

    fn time_interval(&self) -> u32 {
        read_u32_from_bytes(&self.data, 18).unwrap_or(0)
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

    fn statistical_process_type(&self) -> Option<super::tables::TypeOfStatisticalProcessing> {
        None
    }
}
