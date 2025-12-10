use crate::templates::template::{Template, TemplateType};
use crate::utils::{read_u16_from_bytes, read_u32_from_bytes};
use chrono::prelude::*;

use super::product_template::ProductTemplate;
use super::tables::{
    EnsembleForecastType, FixedSurfaceType, GeneratingProcess, TimeUnit,
    TypeOfStatisticalProcessing,
};

/// Product Definition Template 4.11
/// Individual ensemble forecast, control and perturbed, at a horizontal level
/// or in a horizontal layer in a continuous or non-continuous time interval
pub struct EnsembleForecastTimeIntervalTemplate {
    data: Vec<u8>,
    discipline: u8,
}

impl Template for EnsembleForecastTimeIntervalTemplate {
    fn data(&self) -> &[u8] {
        &self.data
    }

    fn template_number(&self) -> u16 {
        11
    }

    fn template_type(&self) -> TemplateType {
        TemplateType::Product
    }

    fn template_name(&self) -> &str {
        "Individual ensemble forecast, control and perturbed, at a horizontal level or in a horizontal layer in a continuous or non-continuous time interval"
    }
}

impl EnsembleForecastTimeIntervalTemplate {
    pub fn new(data: Vec<u8>, discipline: u8) -> Self {
        Self { data, discipline }
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

    pub fn forecast_time(&self) -> u32 {
        read_u32_from_bytes(&self.data, 18).unwrap_or(0)
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

    // Ensemble-specific fields (octets 35-37)
    pub fn type_of_ensemble_forecast(&self) -> EnsembleForecastType {
        self.data[34].into()
    }

    pub fn perturbation_number(&self) -> u8 {
        self.data[35]
    }

    pub fn number_of_forecasts_in_ensemble(&self) -> u8 {
        self.data[36]
    }

    // Time interval fields (octets 38+)
    pub fn end_year(&self) -> u16 {
        read_u16_from_bytes(&self.data, 37).unwrap_or(0)
    }

    pub fn end_month(&self) -> u8 {
        self.data[39]
    }

    pub fn end_day(&self) -> u8 {
        self.data[40]
    }

    pub fn end_hour(&self) -> u8 {
        self.data[41]
    }

    pub fn end_minute(&self) -> u8 {
        self.data[42]
    }

    pub fn end_second(&self) -> u8 {
        self.data[43]
    }

    pub fn number_of_time_ranges(&self) -> u8 {
        self.data[44]
    }

    pub fn number_of_missing_values(&self) -> u32 {
        read_u32_from_bytes(&self.data, 45).unwrap_or(0)
    }

    pub fn statistical_process_type(&self) -> TypeOfStatisticalProcessing {
        if self.data.len() > 49 {
            self.data[49].into()
        } else {
            TypeOfStatisticalProcessing::Missing
        }
    }

    pub fn time_increment_type(&self) -> u8 {
        if self.data.len() > 50 {
            self.data[50]
        } else {
            255
        }
    }

    pub fn time_range_unit(&self) -> TimeUnit {
        if self.data.len() > 51 {
            self.data[51].into()
        } else {
            TimeUnit::Hour
        }
    }

    pub fn time_range_length(&self) -> u32 {
        if self.data.len() > 52 {
            read_u32_from_bytes(&self.data, 52).unwrap_or(0)
        } else {
            0
        }
    }

    pub fn time_increment_unit(&self) -> TimeUnit {
        if self.data.len() > 56 {
            self.data[56].into()
        } else {
            TimeUnit::Hour
        }
    }

    pub fn time_increment(&self) -> u32 {
        if self.data.len() > 57 {
            read_u32_from_bytes(&self.data, 57).unwrap_or(0)
        } else {
            0
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

impl ProductTemplate for EnsembleForecastTimeIntervalTemplate {
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
        if self.data.len() > 56 {
            Some(self.data[56].into())
        } else {
            None
        }
    }

    fn time_interval(&self) -> u32 {
        read_u32_from_bytes(&self.data, 18).unwrap_or(0)
    }

    fn time_increment_interval(&self) -> Option<u32> {
        if self.data.len() > 57 {
            Some(read_u32_from_bytes(&self.data, 57).unwrap_or(0))
        } else {
            None
        }
    }

    fn forecast_datetime(&self, reference_date: DateTime<Utc>) -> DateTime<Utc> {
        let offset_duration = self.time_interval_duration();
        reference_date + offset_duration
    }

    fn forecast_end_datetime(&self, reference_date: DateTime<Utc>) -> Option<DateTime<Utc>> {
        // Use the end time from the template
        let year = self.end_year() as i32;
        let month = self.end_month() as u32;
        let day = self.end_day() as u32;
        let hour = self.end_hour() as u32;
        let minute = self.end_minute() as u32;
        let second = self.end_second() as u32;

        if year > 0 {
            Utc.with_ymd_and_hms(year, month, day, hour, minute, second)
                .single()
        } else {
            // Fall back to calculating from time range
            let time_range_duration = self.time_range_unit().duration(self.time_range_length() as i64);
            Some(reference_date + self.time_interval_duration() + time_range_duration)
        }
    }

    fn first_fixed_surface_type(&self) -> FixedSurfaceType {
        self.data[22].into()
    }

    fn first_fixed_surface_value(&self) -> Option<f64> {
        EnsembleForecastTimeIntervalTemplate::scale_value(
            self.first_fixed_surface_scale_factor(),
            self.first_fixed_surface_scaled_value(),
        )
    }

    fn second_fixed_surface_type(&self) -> FixedSurfaceType {
        self.data[28].into()
    }

    fn second_fixed_surface_value(&self) -> Option<f64> {
        EnsembleForecastTimeIntervalTemplate::scale_value(
            self.second_fixed_surface_scale_factor(),
            self.second_fixed_surface_scaled_value(),
        )
    }

    fn derived_forecast_type(&self) -> Option<super::tables::DerivedForecastType> {
        None
    }

    fn statistical_process_type(&self) -> Option<super::tables::TypeOfStatisticalProcessing> {
        if self.data.len() > 49 {
            Some(self.data[49].into())
        } else {
            None
        }
    }

    fn perturbation_number(&self) -> Option<u8> {
        Some(self.perturbation_number())
    }

    fn number_of_ensemble_members(&self) -> Option<u8> {
        Some(self.number_of_forecasts_in_ensemble())
    }
}
