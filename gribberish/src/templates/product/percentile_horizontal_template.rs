use crate::templates::template::{Template, TemplateType};
use crate::utils::{read_u16_from_bytes, read_u32_from_bytes};
use chrono::{DateTime, Utc};

use super::product_template::ProductTemplate;
use super::tables::{FixedSurfaceType, GeneratingProcess, TimeUnit};
use super::HorizontalAnalysisForecastTemplate;

/// Product Definition Template 4.6
/// Percentile forecasts at a horizontal level or in a horizontal layer
/// at a point in time.
///
/// Identical to template 4.0 (analysis or forecast at a point in time) up to
/// and including the second fixed surface, with a single extra octet carrying
/// the percentile value (1-99) appended at octet 35.
pub struct PercentileHorizontalTemplate {
    data: Vec<u8>,
    discipline: u8,
}

impl Template for PercentileHorizontalTemplate {
    fn data(&self) -> &[u8] {
        &self.data
    }

    fn template_number(&self) -> u16 {
        6
    }

    fn template_type(&self) -> TemplateType {
        TemplateType::Product
    }

    fn template_name(&self) -> &str {
        "Percentile forecasts at a horizontal level or in a horizontal layer at a point in time"
    }
}

impl PercentileHorizontalTemplate {
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

    pub fn observation_cutoff_hours_after_reference_time(&self) -> u16 {
        read_u16_from_bytes(&self.data, 14).unwrap_or(0)
    }

    // Octet 35: percentile value (1-99)
    pub fn percentile_value(&self) -> u8 {
        self.data.get(34).copied().unwrap_or(0)
    }
}

impl ProductTemplate for PercentileHorizontalTemplate {
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

    fn percentile_value(&self) -> Option<u8> {
        Some(self.data.get(34).copied().unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use crate::sections::product_definition::ProductDefinitionSection;

    /// Build a minimal Section 4 byte buffer for template 4.6 (percentile
    /// forecast at a point in time) carrying the given category, parameter
    /// number and percentile value.
    fn pdt6_section(category: u8, parameter: u8, percentile: u8) -> Vec<u8> {
        let mut data = vec![0u8; 35];
        // octets 1-4: section length
        data[0..4].copy_from_slice(&35u32.to_be_bytes());
        // octet 5: section number
        data[4] = 4;
        // octets 8-9: product definition template number = 6
        data[7..9].copy_from_slice(&6u16.to_be_bytes());
        // octet 10: parameter category
        data[9] = category;
        // octet 11: parameter number
        data[10] = parameter;
        // octet 12: generating process (forecast)
        data[11] = 2;
        // octet 18: indicator of unit of time range (hour)
        data[17] = 1;
        // octets 19-22: forecast time
        data[18..22].copy_from_slice(&1u32.to_be_bytes());
        // octet 23: type of first fixed surface (ground or water surface)
        data[22] = 1;
        // octet 29: type of second fixed surface (missing)
        data[28] = 255;
        // octet 35: percentile value
        data[34] = percentile;
        data
    }

    #[test]
    fn parses_percentile_value_and_parameter() {
        // discipline 0, category 19, parameter 239 = CWASP, 90th percentile.
        let data = pdt6_section(19, 239, 90);
        let section = ProductDefinitionSection::from_data(&data);
        assert_eq!(section.product_definition_template_number(), 6);

        let template = section
            .product_definition_template(0)
            .expect("template 4.6 should be supported");

        assert_eq!(template.category_value(), 19);
        assert_eq!(template.parameter_value(), 239);
        assert_eq!(template.percentile_value(), Some(90));

        let parameter = template.parameter().expect("CWASP should resolve");
        assert_eq!(parameter.abbrev, "CWASP");
    }
}
