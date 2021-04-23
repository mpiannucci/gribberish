use crate::{templates::template::{Template, TemplateType}, utils::{grib_power, jpeg}};
use super::data_representation_template::DataRepresentationTemplate;
use super::tables::{CompressionType, OriginalFieldValue};
use crate::unwrap_or_return;
use crate::utils::{from_bits, read_f32_from_bytes, read_i16_from_bytes, bits_to_bytes};
use num::Float;
use openjpeg_sys::opj_stream_set_user_data;
use std::{convert::TryInto, ops::Range, ptr::null_mut};
use std::io::BufReader;

pub struct JPEGDataRepresentationTemplate<'a> {
    data: &'a [u8],
}

impl<'a> Template for JPEGDataRepresentationTemplate<'a> {
    fn data(&self) -> &[u8] {
        self.data
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

impl<'a> JPEGDataRepresentationTemplate<'a> {
    pub fn new(data: &'a [u8]) -> JPEGDataRepresentationTemplate {
        JPEGDataRepresentationTemplate { data }
    }

    pub fn reference_value(&self) -> f32 {
        read_f32_from_bytes(self.data, 11).unwrap_or(0.0)
    }

    pub fn binary_scale_factor(&self) -> i16 {
        read_i16_from_bytes(self.data, 15).unwrap_or(0)
    }

    pub fn decimal_scale_factor(&self) -> i16 {
        read_i16_from_bytes(self.data, 17).unwrap_or(0)
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

impl<'a> DataRepresentationTemplate<f64> for JPEGDataRepresentationTemplate<'a> {
	fn bit_count_per_datapoint(&self) -> usize {
		self.bit_count() as usize
    }
	
	fn unpack_range(&self, bits: Vec<u8>, range: Range<usize>) -> Result<Vec<f64>, String> {

        let bytes = bits_to_bytes(bits).unwrap();

        let bscale = grib_power(self.binary_scale_factor().into(), 2);
        let dscale = grib_power(-(self.decimal_scale_factor() as i32), 10);
        let reference_value: f64 = self.reference_value().into();

        let output_value: Vec<f64> = jpeg::extract_jpeg_data(&bytes)?
            [range]
            .iter()
            .map(|d| {
                ((*d as f64) * bscale + reference_value) * dscale
            })
            .collect();

        Ok(output_value)
	}

    fn unpack_all(&self, bits: Vec<u8>) -> Result<Vec<f64>, String> {
		let bit_count = bits.len();
		self.unpack_range(bits, 0..bit_count)
    }
}