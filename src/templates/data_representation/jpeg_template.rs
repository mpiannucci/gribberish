use crate::{templates::template::{Template, TemplateType}, utils::{grib_power, extract_jpeg_data}};
use super::data_representation_template::DataRepresentationTemplate;
use super::tables::{CompressionType, OriginalFieldValue};
use crate::utils::{from_bits, read_f32_from_bytes, read_i16_from_bytes, bits_to_bytes};
use std::{convert::TryInto, ops::Range, ptr::null_mut};
use std::io::BufReader;

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
        read_i16_from_bytes(self.data.as_slice(), 15).unwrap_or(0)
    }

    pub fn decimal_scale_factor(&self) -> i16 {
        read_i16_from_bytes(self.data.as_slice(), 17).unwrap_or(0)
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
	fn bit_count_per_datapoint(&self) -> usize {
		self.bit_count() as usize
    }
	
	fn unpack(&self, bits: Vec<u8>, range: Range<usize>) -> Result<Vec<f64>, String> {
        let bytes = bits_to_bytes(bits).unwrap();

        let bscale = grib_power(self.binary_scale_factor().into(), 2);
        let dscale = grib_power(-(self.decimal_scale_factor() as i32), 10);
        let reference_value: f64 = self.reference_value().into();

        let output_value: Vec<f64> = extract_jpeg_data(&bytes)?
            [range]
            .iter()
            .map(|d| {
                ((*d as f64) * bscale + reference_value) * dscale
            })
            .collect();

        Ok(output_value)
	}
}
