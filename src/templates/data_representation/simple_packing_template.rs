use super::data_representation_template::DataRepresentationTemplate;
use super::tables::OriginalFieldValue;
use crate::utils::{from_bits, read_f32_from_bytes};
use crate::{
    templates::template::{Template, TemplateType},
    utils::{iter::ScaleGribValueIterator, read_u16_from_bytes},
};

pub struct SimplePackingDataRepresentationTemplate {
    data: Vec<u8>,
}

impl Template for SimplePackingDataRepresentationTemplate {
    fn data(&self) -> &[u8] {
        self.data.as_slice()
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

impl SimplePackingDataRepresentationTemplate {
    pub fn new(data: Vec<u8>) -> SimplePackingDataRepresentationTemplate {
        SimplePackingDataRepresentationTemplate { data }
    }

    pub fn reference_value(&self) -> f32 {
        read_f32_from_bytes(self.data.as_slice(), 11).unwrap_or(0.0)
    }

    pub fn binary_scale_factor(&self) -> i16 {
        as_signed!(
            read_u16_from_bytes(self.data.as_slice(), 15).unwrap_or(0),
            i16
        )
    }

    pub fn decimal_scale_factor(&self) -> i16 {
        as_signed!(
            read_u16_from_bytes(self.data.as_slice(), 17).unwrap_or(0),
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

impl DataRepresentationTemplate<f64> for SimplePackingDataRepresentationTemplate {
    fn compression_type(&self) -> String {
        "Simple Grid Packing".into()
    }

    fn bit_count_per_datapoint(&self) -> usize {
        self.bit_count() as usize
    }

    fn unpack(&self, bits: Vec<u8>) -> Result<Vec<f64>, String> {
        let bits_per_val: usize = self.bit_count().into();
        if bits_per_val == 0 {
            return Err("Invalid bits per value size of 0".into());
        }

        let bit_start_index: usize = 32 - bits_per_val;

        let mut val_bits: [u8; 32] = [0; 32];

        let values = (0..bits.len())
            .step_by(bits_per_val)
            .map(|i| {
                val_bits = [0; 32];

                let mut i_end_index = i + bits_per_val;
                if i_end_index >= bits.len() {
                    i_end_index = bits.len() - 1;
                }
    
                let relevent_bits = &bits[i..i_end_index];
                for (j, bit) in relevent_bits.iter().enumerate() {
                    val_bits[j + bit_start_index] = *bit;
                }

                from_bits::<u32>(&val_bits).unwrap()
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
