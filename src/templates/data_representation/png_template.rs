use std::ops::Range;

use crate::{templates::template::{Template, TemplateType}, utils::{bits_to_bytes, grib_power, read_f32_from_bytes, read_i16_from_bytes, read_u16_from_bytes, read_u32_from_bytes}};
use super::{DataRepresentationTemplate, tables::OriginalFieldValue};
use png::Decoder;

pub struct PNGDataRepresentationTemplate {
    data: Vec<u8>,
}

impl Template for PNGDataRepresentationTemplate {
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
        "grid point data - PNG compression"
    }
}

impl PNGDataRepresentationTemplate {
    pub fn new(data: Vec<u8>) -> PNGDataRepresentationTemplate {
        PNGDataRepresentationTemplate { data }
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
}

impl DataRepresentationTemplate<f64> for PNGDataRepresentationTemplate {
	fn bit_count_per_datapoint(&self) -> usize {
		self.bit_count() as usize
    }

    fn unpack(&self, bits: Vec<u8>, range: Range<usize>) -> Result<Vec<f64>, String> {
        let bytes = bits_to_bytes(bits).unwrap();
        
        let decoder = Decoder::new(bytes.as_slice());
        let mut reader = decoder.read_info().unwrap();
        
        let mut image_data: Vec<u8> = vec![0; reader.output_buffer_size()];
        let _ = reader.next_frame(&mut image_data).unwrap();

        let bscale = grib_power(self.binary_scale_factor().into(), 2);
        let dscale = grib_power(-(self.decimal_scale_factor() as i32), 10);
        let reference_value: f64 = self.reference_value().into();

        let bytes_per_datapoint = self.bit_count_per_datapoint() / 8;
        let start_byte = range.start * bytes_per_datapoint;
        let end_byte = range.end * bytes_per_datapoint;

        let mut out = Vec::new();

        // TODO: support non greyscale imagery
        for i in (start_byte..end_byte).step_by(bytes_per_datapoint) {
            if let Some(data_point) = read_u16_from_bytes(&image_data, i) {
                let scaled = ((data_point as f64) * bscale + reference_value) * dscale;
                out.push(scaled);
            } else {
                out.push(f64::NAN);
            }
        }

        Ok(out)
    }
}