use crate::templates::template::{Template, TemplateType};
use super::data_representation_template::DataRepresentationTemplate;
use super::tables::{OriginalFieldValue};
use crate::unwrap_or_return;
use crate::utils::{from_bits, read_f32_from_bytes, read_i16_from_bytes};
use num::Float;
use std::ops::Range;

pub struct SimpleGridPointDataRepresentationTemplate<'a> {
    data: &'a [u8],
}

impl<'a> Template for SimpleGridPointDataRepresentationTemplate<'a> {
    fn data(&self) -> &[u8] {
        self.data
    }

    fn template_number(&self) -> u16 {
        0
    }

    fn template_type(&self) -> TemplateType {
        TemplateType::DataRepresentation
    }

    fn template_name(&self) -> &str {
        "grid point data - simple packing"
    }
}

impl<'a> SimpleGridPointDataRepresentationTemplate<'a> {
    pub fn new(data: &'a [u8]) -> SimpleGridPointDataRepresentationTemplate {
        SimpleGridPointDataRepresentationTemplate { data }
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
}

impl<'a> DataRepresentationTemplate<f64> for SimpleGridPointDataRepresentationTemplate<'a> {
	fn bit_count_per_datapoint(&self) -> usize {
		self.bit_count() as usize
    }

    fn scaled_value(&self, raw_value: f64) -> f64 {
        let reference_value: f64 = self.reference_value().into();
        let binary_scale_factor: i32 = self.binary_scale_factor().into();
        let decimal_scale_factor: i32 = self.decimal_scale_factor().into();

        (reference_value + (raw_value * 2.0.powi(binary_scale_factor)))
            / 10.0.powi(decimal_scale_factor)
    }
	
	fn unpack_range(&self, bits: Vec<u8>, range: Range<usize>) -> Result<Vec<f64>, String> {
        let mut v = Vec::new();

        let bits_per_val: usize = self.bit_count().into();
        if bits_per_val == 0 {
            return Err("Invalid bits per value size of 0".into());
        }

        let bit_start_index: usize = 32 - bits_per_val;

        let mut raw_value: f64 = 0.0;
		let mut val_bits: [u8; 32] = [0; 32];
		
		let start_index = range.start * bits_per_val;
		let end_index = range.end * bits_per_val;

        for i in (start_index..end_index).step_by(bits_per_val) {
            val_bits = [0; 32];

            let mut i_end_index = i + bits_per_val;
            if i >= bits.len() {
                continue;
            } else if i_end_index >= bits.len() {
                i_end_index = bits.len() - 1;
            }

            let relevent_bits = &bits[i..i_end_index];
            for (j, bit) in relevent_bits.iter().enumerate() {
                val_bits[j + bit_start_index] = *bit;
            }

            raw_value = unwrap_or_return!(
                from_bits::<u32>(&val_bits),
                "failed to convert value to u32".into()
            )
            .into();
            let val = self.scaled_value(raw_value);
            v.push(val);
        }

        Ok(v)
	}

    fn unpack_all(&self, bits: Vec<u8>) -> Result<Vec<f64>, String> {
		let bit_count = bits.len();
		self.unpack_range(bits, 0..bit_count)
    }
}
