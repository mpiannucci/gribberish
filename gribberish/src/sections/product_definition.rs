use crate::{utils::{read_u16_from_bytes, read_u32_from_bytes}, templates::product::{product_template::ProductTemplate, HorizontalAnalysisForecastTemplate, AverageAccumulationExtremeHorizontalAnalysisForecastTemplate, DerivedEnsembleHorizontalAnalysisForecastTemplate, derived_ensemble_horizontal_forecast_time_interval_template::DerivedEnsembleHorizontalForecastTimeIntervalTemplate}};
use super::grib_section::GribSection;

pub struct ProductDefinitionSection<'a> {
    data: &'a [u8],
}

impl <'a> ProductDefinitionSection<'a> {
    pub fn from_data(data: &'a [u8]) -> Self {
        ProductDefinitionSection {
            data,
        }
    }

    pub fn coord_values_after_template(&self) -> u16 {
        read_u16_from_bytes(self.data, 5).unwrap_or(0)
    }

    pub fn product_definition_template_number(&self) -> u16 {
        read_u16_from_bytes(self.data, 7).unwrap_or(0)
    }

    pub fn product_definition_template(&self, discipline: u8) -> Option<Box<dyn ProductTemplate>> {
        match self.product_definition_template_number() {
            0 => Some(Box::new(HorizontalAnalysisForecastTemplate::new(self.data.to_vec(), discipline))),
            2 => Some(Box::new(DerivedEnsembleHorizontalAnalysisForecastTemplate::new(self.data.to_vec(), discipline))),
            8 => Some(Box::new(AverageAccumulationExtremeHorizontalAnalysisForecastTemplate::new(self.data.to_vec(), discipline))),
            12 => Some(Box::new(DerivedEnsembleHorizontalForecastTimeIntervalTemplate::new(self.data.to_vec(), discipline))),
            _ => None
        }
    }
}

impl <'a> GribSection for ProductDefinitionSection<'a> {
    fn len(&self) -> usize {
        read_u32_from_bytes(&self.data[0..4], 0).unwrap_or(0) as usize
    }

    fn number(&self) -> u8 {
        self.data[4]
    }
}