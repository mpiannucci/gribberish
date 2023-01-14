use crate::{templates::template::{Template, TemplateType}, utils::{bits_to_bytes, read_f32_from_bytes, read_u16_from_bytes, iter::ScaleGribValueIterator}};
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
}

impl DataRepresentationTemplate<f64> for PNGDataRepresentationTemplate {
    fn compression_type(&self) -> String {
        "PNG".into()
    }

	fn bit_count_per_datapoint(&self) -> usize {
		self.bit_count() as usize
    }

    fn unpack(&self, bits: Vec<u8>) -> Result<Vec<f64>, String> {
        let bytes = bits_to_bytes(bits).unwrap();
        
        let decoder = Decoder::new(bytes.as_slice());
        let mut reader = decoder.read_info().unwrap();
        
        let mut image_data: Vec<u8> = vec![0; reader.output_buffer_size()];
        let _ = reader.next_frame(&mut image_data).unwrap();

        let bytes_per_datapoint = self.bit_count_per_datapoint() / 8;

        let values = (0..image_data.len())
            .step_by(bytes_per_datapoint)
            .map(|ib| read_u16_from_bytes(&image_data, ib).unwrap())
            .scale_value_by(self.binary_scale_factor(), self.decimal_scale_factor(), self.reference_value())
            .collect();

        Ok(values)
    }
}