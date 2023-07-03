use crate::templates::template::{Template, TemplateType};
use crate::utils::{read_u16_from_bytes, read_u32_from_bytes};
use chrono::{prelude::*, Duration};

use super::HorizontalAnalysisForecastTemplate;
use super::product_template::ProductTemplate;
use super::tables::{TypeOfStatisticalProcessing, TypeOfTimeInterval, TimeUnit, GeneratingProcess, FixedSurfaceType, DerivedForecastType};

pub struct DerivedEnsembleHorizontalForecastTimeIntervalTemplate {
	data: Vec<u8>,
	discipline: u8,
}

impl Template for DerivedEnsembleHorizontalForecastTimeIntervalTemplate {
	fn data(&self) -> &[u8] {
    	&self.data
 	}

 	fn template_number(&self) -> u16 {
 	    12
 	}

 	fn template_type(&self) -> TemplateType {
 	    TemplateType::Product
 	}
 	
    fn template_name(&self) -> &str {
        "Derived forecasts based on all ensemble members at a horizontal level
        or in a horizontal layer in a continuous or non-continuous time interval"
    }
}

impl DerivedEnsembleHorizontalForecastTimeIntervalTemplate {
	pub fn new(	data: Vec<u8>, discipline: u8) -> Self {
		Self {
            data, 
            discipline,
        }
	}

	pub fn category_value(&self) -> u8 {
		self.data[9]
	}

	pub fn parameter_value(&self) -> u8{
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

	pub fn number_of_forecasts_in_ensemble(&self) -> u8 {
		self.data[35]
	}

    pub fn time_interval_end(&self) -> DateTime<Utc>  {
        let data = self.data();
        let year = read_u16_from_bytes(data, 36).unwrap_or(0) as i32;
        let month = data[38] as u32;
        let day = data[39] as u32;
        let hour = data[40] as u32;
        let minute = data[41] as u32;
        let second = data[42] as u32;

        Utc.with_ymd_and_hms(year as i32, month, day, hour, minute, second).unwrap()
    }

    pub fn number_of_time_ranges(&self) -> u8 {
        self.data()[43]
    }

    pub fn number_of_values_missing_from_stats(&self) -> u32 {
        read_u32_from_bytes(self.data(), 44).unwrap_or(0)
    }

    pub fn type_of_time_interval(&self) -> TypeOfTimeInterval {
        self.data()[49].into()
    }

    pub fn statistical_process_time_unit(&self) -> TimeUnit {
        self.data()[50].into()
    }

    pub fn statistical_process_time_interval(&self) -> u32 {
        read_u32_from_bytes(self.data(), 51).unwrap_or(0)
    }

    pub fn time_increment_unit(&self) -> TimeUnit {
        self.data()[56].into()
    }

    pub fn time_increment_interval(&self) -> u32 {
        read_u32_from_bytes(self.data(), 56).unwrap_or(0)
    }
}

impl ProductTemplate for DerivedEnsembleHorizontalForecastTimeIntervalTemplate {
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

    fn forecast_datetime(&self, reference_date: DateTime<Utc>) -> DateTime<Utc> {
        let forecast_offset = self.forecast_time();
		let offset_duration: Duration = self.time_unit().duration(forecast_offset as i64);
		reference_date + offset_duration
    }

    fn first_fixed_surface_type(&self) -> FixedSurfaceType {
        self.data[22].into()
    }

    fn first_fixed_surface_value(&self) -> Option<f64> {
        HorizontalAnalysisForecastTemplate::scale_value(self.first_fixed_surface_scale_factor(), self.first_fixed_surface_scaled_value())
    }

    fn second_fixed_surface_type(&self) -> FixedSurfaceType {
        self.data[28].into()
    }

    fn second_fixed_surface_value(&self) -> Option<f64> {
        HorizontalAnalysisForecastTemplate::scale_value(self.second_fixed_surface_scale_factor(), self.second_fixed_surface_scaled_value())
    }

    fn derived_forecast_type(&self) -> Option<DerivedForecastType> {
        Some(self.data[34].into())
    }

    fn statistical_process_type(&self) -> Option<TypeOfStatisticalProcessing> {
        Some(self.data()[48].into())
    }

    fn end_time(&self, _reference_date: DateTime<Utc>) -> Option<DateTime<Utc>> {
        Some(self.time_interval_end())
    }    
}