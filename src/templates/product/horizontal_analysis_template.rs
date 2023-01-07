use gribberish_types::Parameter;
use crate::templates::template::{Template, TemplateType};
use crate::utils::{read_u16_from_bytes, read_u32_from_bytes};
use chrono::{Utc, DateTime, Duration};

use super::tables::{TimeUnit, GeneratingProcess, FixedSurfaceTypes, oceanographic_category, meteorological_category, multiradar_category, meteorological_parameter, oceanographic_parameter, multiradar_parameter};

pub struct HorizontalAnalysisForecastTemplate<'a> {
	data: &'a[u8],
	discipline: u8,
}

impl <'a> Template for HorizontalAnalysisForecastTemplate<'a> {
	fn data(&self) -> &[u8] {
    	self.data
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

impl <'a> HorizontalAnalysisForecastTemplate<'a> {

	pub fn new(	data: &'a[u8], discipline: u8) -> Self {
		HorizontalAnalysisForecastTemplate { data, discipline }
	}

	pub fn category_value(&self) -> u8 {
		self.data[9]
	}

	pub fn parameter_value(&self) -> u8{
		self.data[10]
	}

	pub fn category(&self) -> &'static str {
		let category = self.category_value();
		match self.discipline {
			0 => meteorological_category(category),
			10 => oceanographic_category(category),
			209 => multiradar_category(category),
			_ => "",
		}
	}

	pub fn parameter(&self) -> Option<Parameter> {
		let category = self.category_value();
		let parameter = self.parameter_value();

		match self.discipline {
			0 => meteorological_parameter(category, parameter),
			10 => oceanographic_parameter(category, parameter),
			209 => multiradar_parameter(category, parameter),
			_ => None,
		}
	}

	pub fn generating_process(&self) -> GeneratingProcess {
		self.data[12].into()
	}

	pub fn observation_cutoff_hours_after_reference_time(&self) -> u16 {
		read_u16_from_bytes(self.data, 14).unwrap_or(0)
	}

	pub fn observation_cutoff_minutes_after_cutoff_time(&self) -> u8 {
		self.data[16]
	}

	pub fn time_unit(&self) -> TimeUnit {
		self.data[17].into()
	}

	pub fn forecast_time(&self) -> u32 {
		read_u32_from_bytes(self.data, 18).unwrap_or(0)
	}

	pub fn forecast_datetime(&self, reference_date: DateTime<Utc>) -> DateTime<Utc> {
		let forecast_offset = self.forecast_time();
		let offset_duration: Duration = self.time_unit().duration(forecast_offset as i64);
		reference_date + offset_duration
	}

    pub fn first_fixed_surface_type(&self) -> FixedSurfaceTypes {
        self.data[22].into()
    }

    pub fn first_fixed_surface_scale_factor(&self) -> u8 {
        self.data[23]
    }

    pub fn first_fixed_surface_scaled_value(&self) -> u32 {
        read_u32_from_bytes(self.data, 24).unwrap_or(0)
    }

	pub fn array_index(&self) -> Option<usize> {
		match self.first_fixed_surface_type() {
			FixedSurfaceTypes::OrderedSequence => Some(self.first_fixed_surface_scaled_value() as usize), 
			_ => None,
		}
	}

    pub fn second_fixed_surface_type(&self) -> FixedSurfaceTypes {
        self.data[28].into()
    }

    pub fn second_fixed_surface_scale_factor(&self) -> u8 {
        self.data[29]
    }

    pub fn second_fixed_surface_scaled_value(&self) -> u32 {
        read_u32_from_bytes(self.data, 30).unwrap_or(0)
    }
}