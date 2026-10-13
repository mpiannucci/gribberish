use bitvec::prelude::*;

use super::{tables::OriginalFieldValue, DataRepresentationTemplate};
use crate::{
    error::GribberishError,
    templates::template::{Template, TemplateType},
    utils::{iter::ScaleGribValueIterator, read_f32_from_bytes, read_u16_from_bytes},
};
use png::{Decoder, Transformations};

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
}

impl DataRepresentationTemplate<f64> for PNGDataRepresentationTemplate {
    fn compression_type(&self) -> String {
        "PNG".into()
    }

    fn bit_count_per_datapoint(&self) -> usize {
        self.bit_count() as usize
    }

    fn unpack(&self, bits: &BitSlice<u8, Msb0>) -> Result<Vec<f64>, GribberishError> {
        let bytes: Vec<u8> = bits.to_bitvec().into();

        let png_error = |e: png::DecodingError| {
            GribberishError::DataRepresentationTemplateError(format!("PNG decode failed: {e}"))
        };

        // GRIB2 template 5.41 packs each point as one big-endian integer spread across the
        // image's samples: 1-8 bits as greyscale, 16 bits as 16-bit greyscale, 24 bits as
        // RGB and 32 bits as RGBA. Expand sub-byte depths so every sample is a whole byte,
        // then size each point from the decoded image rather than assuming two bytes.
        let mut decoder = Decoder::new(bytes.as_slice());
        decoder.set_transformations(Transformations::EXPAND);
        let mut reader = decoder.read_info().map_err(png_error)?;

        let mut image_data: Vec<u8> = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut image_data).map_err(png_error)?;
        let image_data = &image_data[..info.buffer_size()];

        let bytes_per_sample = (info.bit_depth as usize).max(8) / 8;
        let bytes_per_datapoint = bytes_per_sample * info.color_type.samples();

        let values = image_data
            .chunks_exact(bytes_per_datapoint)
            .map(|point| point.iter().fold(0u32, |acc, &b| (acc << 8) | b as u32))
            .scale_value_by(
                self.binary_scale_factor(),
                self.decimal_scale_factor(),
                self.reference_value(),
            )
            .collect();

        Ok(values)
    }
}
