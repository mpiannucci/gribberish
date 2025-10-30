use bitvec::prelude::*;

use crate::{error::GribberishError, utils::iter::ScaleGribValueIterator};
use itertools::izip;

use crate::{
    templates::template::{Template, TemplateType},
    utils::{read_f32_from_bytes, read_u16_from_bytes, read_u32_from_bytes},
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
        as_signed!(
            read_u16_from_bytes(self.data.as_slice(), 15).unwrap_or(0),
            16,
            i16
        )
    }

    pub fn decimal_scale_factor(&self) -> i16 {
        as_signed!(
            read_u16_from_bytes(self.data.as_slice(), 17).unwrap_or(0),
            16,
            i16
        )
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

    fn unpack(&self, bits: &BitSlice<u8, Msb0>) -> Result<Vec<f64>, GribberishError> {
        let ng = self.number_of_groups() as usize;
        let nbits = self.bit_count() as usize;

        let group_references = (0..ng).map(|ig| {
            if nbits == 0 {
                0
            } else {
                let start = ig * nbits;
                bits[start..start + nbits].load::<u32>()
            }
        });

        let group_widths_start = ((ng * nbits) as f32 / 8.0).ceil() as usize * 8;
        let n_width_bits = self.group_width_bits() as usize;
        let group_widths = (0..ng).map(|ig| {
            if n_width_bits == 0 {
                0
            } else {
                let start = group_widths_start + ig * n_width_bits;
                bits[start..start + n_width_bits].load::<u32>()
                    + self.group_width_reference() as u32
            }
        });

        let group_lengths_start =
            group_widths_start + (((n_width_bits * ng) as f32 / 8.0).ceil() as usize * 8);
        let n_length_bits = self.group_length_bits() as usize;
        let group_lengths = (0..ng).map(|ig| {
            if n_length_bits == 0 {
                0
            } else {
                let start = group_lengths_start + ig * n_length_bits;
                bits[start..start + n_length_bits].load::<u32>()
                    * self.group_length_increment() as u32
                    + self.group_length_reference()
            }
        });

        let mut pos =
            group_lengths_start + (((n_length_bits * ng) as f32 / 8.0).ceil() as usize * 8);
        let values = izip!(group_references, group_widths, group_lengths)
        .flat_map(|(reference, width, length)| {
            let n_bits = (width * length) as usize;
            let group_values = (0..length).map(move |i| {
                let value = if width == 0 {
                    0u32
                } else {
                    bits[pos + (i * width) as usize
                        ..pos + (i * width) as usize + width as usize]
                        .load_be::<u32>()
                };
                let raw = as_signed!(value, 32, i32);
                raw + reference as i32
            });

            pos += n_bits;

            group_values
        })
        .scale_value_by(
            self.binary_scale_factor(),
            self.decimal_scale_factor(),
            self.reference_value(),
        )
        .collect();

        Ok(values)
    }
}
