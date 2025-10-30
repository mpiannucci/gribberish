use bitvec::prelude::*;

use crate::{error::GribberishError, templates::template::{Template, TemplateType}, utils::{extract_jpeg_data, iter::ScaleGribValueIterator, read_u16_from_bytes}};
use super::data_representation_template::DataRepresentationTemplate;
use super::tables::{CompressionType, OriginalFieldValue};
use crate::utils::read_f32_from_bytes;

pub struct JPEGDataRepresentationTemplate {
    data: Vec<u8>,
}

impl Template for JPEGDataRepresentationTemplate {
    fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    fn template_number(&self) -> u16 {
        40
    }

    fn template_type(&self) -> TemplateType {
        TemplateType::DataRepresentation
    }

    fn template_name(&self) -> &str {
        "grid point data - jpeg2000 compression"
    }
}

impl JPEGDataRepresentationTemplate {
    pub fn new(data: Vec<u8>) -> JPEGDataRepresentationTemplate {
        JPEGDataRepresentationTemplate { data }
    }

    pub fn reference_value(&self) -> f32 {
        read_f32_from_bytes(self.data.as_slice(), 11).unwrap_or(0.0)
    }

    pub fn binary_scale_factor(&self) -> i16 {
        as_signed!(read_u16_from_bytes(self.data.as_slice(), 15).unwrap_or(0), 16, i16)
    }

    pub fn decimal_scale_factor(&self) -> i16 {
        as_signed!(read_u16_from_bytes(self.data.as_slice(), 17).unwrap_or(0), 16, i16)
    }

    pub fn bit_count(&self) -> u8 {
        self.data[19]
    }

    pub fn original_field_value(&self) -> OriginalFieldValue {
        self.data[20].into()
    }

    pub fn compression_type(&self) -> CompressionType {
        self.data[21].into()
    }

    pub fn compression_ration(&self) -> u8 {
        self.data[22]
    }
}

impl DataRepresentationTemplate<f64> for JPEGDataRepresentationTemplate {
    fn compression_type(&self) -> String {
        "JPEG2000".into()
    }

	fn bit_count_per_datapoint(&self) -> usize {
		self.bit_count() as usize
    }

	fn unpack(&self, bits: &BitSlice<u8, Msb0>) -> Result<Vec<f64>, GribberishError> {
        let bytes = bits.to_bitvec().into();

        let output_value: Vec<f64> = extract_jpeg_data(&bytes)?
            .into_iter()
            .scale_value_by(self.binary_scale_factor(), self.decimal_scale_factor(), self.reference_value())
            .collect();

        Ok(output_value)
	}
}
