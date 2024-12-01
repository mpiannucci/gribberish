use bitvec::prelude::*;

use super::data_representation_template::DataRepresentationTemplate;
use super::tables::OriginalFieldValue;
use crate::utils::read_f32_from_bytes;
use crate::{
    error::GribberishError,
    templates::template::{Template, TemplateType},
    utils::{
        extract_ccsds_data, iter::ScaleGribValueIterator, read_u16_from_bytes, read_u32_from_bytes,
    },
};

pub struct CCSDSDataRepresentationTemplate {
    data: Vec<u8>,
}

impl Template for CCSDSDataRepresentationTemplate {
    fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    fn template_number(&self) -> u16 {
        42
    }

    fn template_type(&self) -> TemplateType {
        TemplateType::DataRepresentation
    }

    fn template_name(&self) -> &str {
        "grid point and spectral data - CCSDS recommended lossless compression"
    }
}

impl CCSDSDataRepresentationTemplate {
    pub fn new(data: Vec<u8>) -> CCSDSDataRepresentationTemplate {
        CCSDSDataRepresentationTemplate { data }
    }

    pub fn data_point_count(&self) -> usize {
        read_u32_from_bytes(self.data.as_slice(), 5).unwrap_or(0) as usize
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

    // Nbits
    pub fn bit_count(&self) -> u8 {
        self.data[19]
    }

    pub fn original_field_value(&self) -> OriginalFieldValue {
        self.data[20].into()
    }

    pub fn ccsds_compression_options_mask(&self) -> u8 {
        self.data[21]
    }

    pub fn block_size(&self) -> u8 {
        self.data[22]
    }

    // restart interval
    pub fn reference_sample_interval(&self) -> u16 {
        read_u16_from_bytes(self.data.as_slice(), 23).unwrap_or(0)
    }
}

impl DataRepresentationTemplate<f64> for CCSDSDataRepresentationTemplate {
    fn compression_type(&self) -> String {
        "CCSDS".into()
    }

    fn bit_count_per_datapoint(&self) -> usize {
        self.bit_count() as usize
    }

    fn unpack(&self, bits: &BitSlice<u8, Msb0>) -> Result<Vec<f64>, GribberishError> {
        let bits_per_val: usize = self.bit_count().into();
        if bits_per_val == 0 {
            return Ok(vec![]);
        }

        let bytes: Vec<u8> = bits.to_bitvec().into();

        let nbytes_per_sample: usize = (bits_per_val + 7) / 8;

        let size = self.data_point_count() * nbytes_per_sample;
        let outputwr = extract_ccsds_data(
            bytes,
            self.block_size(),
            self.ccsds_compression_options_mask(),
            size,
            self.reference_sample_interval(),
            bits_per_val,
        );

        match outputwr {
            Ok(output_value) => {
                // Ok(output_value)
                Ok(output_value
                    .into_iter()
                    .scale_value_by(
                        self.binary_scale_factor(),
                        self.decimal_scale_factor(),
                        self.reference_value(),
                    )
                    .collect())
            }
            Err(e) => Err(e),
        }
    }
}
