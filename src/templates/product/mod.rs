pub mod tables;
pub mod horizontal_analysis_template;

use horizontal_analysis_template::HorizontalAnalysisForecastTemplate;

pub enum ProductTemplate<'a> {
	HorizontalAnalysisForecast(HorizontalAnalysisForecastTemplate<'a>),
	Other,
}

impl <'a> ProductTemplate<'a> {
	pub fn from_template_number(template_number: u16, data: &'a[u8], discipline: u8) -> ProductTemplate {
		match template_number {
			0 => ProductTemplate::HorizontalAnalysisForecast(HorizontalAnalysisForecastTemplate::new(data, discipline)),
			_ => ProductTemplate::Other,
		}
	}
}