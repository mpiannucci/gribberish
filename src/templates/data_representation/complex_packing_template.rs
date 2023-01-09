use itertools::izip;

use crate::{
    templates::template::{Template, TemplateType},
    utils::{from_bits, read_f32_from_bytes, read_u32_from_bytes, read_u16_from_bytes},
};

use super::{
    tables::{GroupSplittingMethod, MissingValueManagement, OriginalFieldValue},
    DataRepresentationTemplate,
};

pub struct ComplexPackingDataRepresentationTemplate {
    data: Vec<u8>,
}

impl Template for ComplexPackingDataRepresentationTemplate {
    fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    fn template_number(&self) -> u16 {
        2
    }

    fn template_type(&self) -> TemplateType {
        TemplateType::DataRepresentation
    }

    fn template_name(&self) -> &str {
        "grid point data - complex packing"
    }
}

impl ComplexPackingDataRepresentationTemplate {
    pub fn new(data: Vec<u8>) -> ComplexPackingDataRepresentationTemplate {
        ComplexPackingDataRepresentationTemplate { data }
    }

    pub fn reference_value(&self) -> f32 {
        read_f32_from_bytes(self.data.as_slice(), 11).unwrap_or(0.0)
    }

    pub fn binary_scale_factor(&self) -> i16 {
        as_signed!(read_u16_from_bytes(self.data.as_slice(), 15).unwrap_or(0), i16)
    }

    pub fn decimal_scale_factor(&self) -> i16 {
        as_signed!(read_u16_from_bytes(self.data.as_slice(), 17).unwrap_or(0), i16)
    }

    pub fn bit_count(&self) -> u8 {
        self.data[19]
    }

    pub fn original_field_value(&self) -> OriginalFieldValue {
        self.data[20].into()
    }

    pub fn group_splitting_method(&self) -> GroupSplittingMethod {
        self.data[21].into()
    }

    pub fn missing_value_management(&self) -> MissingValueManagement {
        self.data[22].into()
    }

    pub fn primary_missing_value_substitute(&self) -> f32 {
        read_f32_from_bytes(self.data.as_slice(), 23).unwrap_or(0.0)
    }

    pub fn secondary_missing_value_substitute(&self) -> f32 {
        read_f32_from_bytes(self.data.as_slice(), 27).unwrap_or(0.0)
    }

    pub fn number_of_groups(&self) -> u32 {
        read_u32_from_bytes(self.data.as_slice(), 31).unwrap()
    }

    pub fn group_width_reference(&self) -> u8 {
        self.data[35]
    }

    pub fn group_width_bits(&self) -> u8 {
        self.data[36]
    }

    pub fn group_length_reference(&self) -> u32 {
        read_u32_from_bytes(self.data.as_slice(), 37).unwrap()
    }

    pub fn group_length_increment(&self) -> u8 {
        self.data[41]
    }

    pub fn group_last_length(&self) -> u32 {
        read_u32_from_bytes(self.data.as_slice(), 42).unwrap()
    }

    pub fn group_length_bits(&self) -> u8 {
        self.data[46]
    }
}

impl DataRepresentationTemplate<f64> for ComplexPackingDataRepresentationTemplate {
    fn compression_type(&self) -> String {
        "Complex Grid Packing".into()
    }

    fn bit_count_per_datapoint(&self) -> usize {
        self.bit_count() as usize
    }

    fn unpack(&self, bits: Vec<u8>, range: std::ops::Range<usize>) -> Result<Vec<f64>, String> {
        let ng = self.number_of_groups() as usize;
        let nbits = self.bit_count() as usize;

        let group_reference_start_index = 32 - nbits;
        let group_references = (0..ng).map(|ig| {
            let start = ig * nbits;
            let mut temp_container: [u8; 32] = [0; 32];
            for i in 0..nbits {
                temp_container[group_reference_start_index + i] = bits[start + i];
            }

            from_bits::<u32>(&temp_container).unwrap()
        });

        let group_widths_start = ((ng * nbits) as f32 / 8.0).ceil() as usize * 8;
        let n_width_bits = self.group_width_bits() as usize;
        let group_width_start_index = 32 - n_width_bits;
        let group_widths = (0..ng).map(|ig| {
            let start = group_widths_start + ig * n_width_bits;
            let mut temp_container: [u8; 32] = [0; 32];
            for i in 0..nbits {
                temp_container[group_width_start_index + i] = bits[start + i];
            }

            from_bits::<u32>(&temp_container).unwrap() + self.group_width_reference() as u32
        });

        let group_lengths_start = group_widths_start + (((n_width_bits * ng) as f32 / 8.0).ceil() as usize * 8);
        let n_length_bits = self.group_length_bits() as usize;
        let group_length_start_index = 32 - n_length_bits;
        let group_lengths = (0..ng).map(|ig| {
            let start = group_lengths_start + ig * n_length_bits;
            let mut temp_container: [u8; 32] = [0; 32];
            for i in 0..nbits {
                temp_container[group_length_start_index + i] = bits[start + i];
            }

            from_bits::<u32>(&temp_container).unwrap() * self.group_length_increment() as u32
                + self.group_length_reference()
        });

        let bscale = 2_f64.powi(self.binary_scale_factor().into());
        let dscale = 10_f64.powi(-self.decimal_scale_factor() as i32);
        let reference_value: f64 = self.reference_value().into();

        let mut pos = group_lengths_start + (((n_length_bits * ng) as f32 / 8.0).ceil() as usize * 8);
        let mut raw_values = Vec::with_capacity(ng);
        for (reference, width, length) in izip!(group_references, group_widths, group_lengths) {
            if width == 0 {
                raw_values.push(vec![reference as i32; length as usize]);
                continue;
            }
            
            let n_bits = (width * length) as usize;
            let bit_start_index = 32 - width as usize;
            let mut temp_container: [u8; 32] = [0; 32];
            let group_values: Vec<i32> = (0..length)
                .map(|i| {
                    temp_container = [0; 32];
                    for bit in 0..width as usize {
                        temp_container[bit_start_index + bit as usize] = bits[pos + (i * width) as usize + bit];
                    }

                    let raw = from_bits::<u32>(&temp_container).unwrap(); 
                    as_signed!(raw, i32) + reference as i32
                })
                .collect();

            pos += n_bits;

            raw_values.push(group_values);
        }
        let values: Vec<f64> = raw_values
        .iter()
        .flatten()
        .map(|v| (*v as f64 * bscale + reference_value) * dscale)
        .collect();

        Ok(values[range].to_vec())
    }
}
