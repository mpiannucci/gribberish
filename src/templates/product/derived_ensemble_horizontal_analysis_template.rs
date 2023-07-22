use crate::templates::template::{Template, TemplateType};
use crate::utils::{read_u32_from_bytes};
use chrono::{Utc, DateTime, Duration};

use super::product_template::ProductTemplate;
use super::tables::{TimeUnit, GeneratingProcess, FixedSurfaceType, DerivedForecastType};

pub struct DerivedEnsembleHorizontalAnalysisForecastTemplate {
	data: Vec<u8>,
	discipline: u8,
}

impl Template for DerivedEnsembleHorizontalAnalysisForecastTemplate {
	fn data(&self) -> &[u8] {
    	&self.data
 	}

 	fn template_number(&self) -> u16 {
 	    2
 	}

 	fn template_type(&self) -> TemplateType {
 	    TemplateType::Product
 	}
 	
    fn template_name(&self) -> &str {
        "Derived forecast, based on all ensemble members at a horizontal
		level or in a horizontal layer at a point in time"
    }
}

impl DerivedEnsembleHorizontalAnalysisForecastTemplate {

	pub fn new(	data: Vec<u8>, discipline: u8) -> Self {
		DerivedEnsembleHorizontalAnalysisForecastTemplate { data, discipline }
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

	pub fn array_index(&self) -> Option<usize> {
		match self.first_fixed_surface_type() {
			FixedSurfaceType::OrderedSequence => Some(self.first_fixed_surface_scaled_value() as usize), 
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

impl ProductTemplate for DerivedEnsembleHorizontalAnalysisForecastTemplate {
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
        DerivedEnsembleHorizontalAnalysisForecastTemplate::scale_value(self.first_fixed_surface_scale_factor(), self.first_fixed_surface_scaled_value())
    }

    fn second_fixed_surface_type(&self) -> FixedSurfaceType {
        self.data[28].into()
    }

    fn second_fixed_surface_value(&self) -> Option<f64> {
        DerivedEnsembleHorizontalAnalysisForecastTemplate::scale_value(self.second_fixed_surface_scale_factor(), self.second_fixed_surface_scaled_value())
    }

    fn derived_forecast_type(&self) -> Option<DerivedForecastType> {
        Some(self.data[34].into())
    }

    fn statistical_process_type(&self) -> Option<super::tables::TypeOfStatisticalProcessing> {
        None
    }

    fn time_interval_end(&self) -> Option<DateTime<Utc>> {
        None
    }
}