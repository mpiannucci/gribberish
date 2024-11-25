use bitvec::prelude::*;

use crate::{error::GribberishError, templates::template::{Template, TemplateType}, utils::{iter::ScaleGribValueIterator, read_u16_from_bytes, extract_ccsds_data}};
use super::data_representation_template::DataRepresentationTemplate;
use super::tables::{CompressionType, OriginalFieldValue};
use crate::utils::read_f32_from_bytes;


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
        "grid point data - ccsds compression"
    }
}

impl CCSDSDataRepresentationTemplate {
    pub fn new(data: Vec<u8>) -> CCSDSDataRepresentationTemplate {
        CCSDSDataRepresentationTemplate { data }
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

    // pub fn compression_type(&self) -> CompressionType {
    //     self.data[21].into()
    // }

    // pub fn compression_ration(&self) -> u8 {
    //     self.data[22]
    // }
    /**
     * 22 			CCSDS compression options mask (See Note 3)
23 			Block size
24-25 			Reference sample interval 
     */
    // pub fn ccsds_compression_options_mask(&self) -> u8 {
    //     self.data[21]
    // }

    // pub fn block_size(&self) -> u8 {
    //     self.data[22]
    // }
    pub fn ccsds_compression_options_mask(&self) -> u8 {
        self.data[21]
    }

    pub fn block_size(&self) -> u8 {
        self.data[22]
    }

    pub fn reference_sample_interval(&self) -> u16 {
        // read_u16_from_bytes(self.data.as_slice(), 24).unwrap_or(0)
        self.data[24] as u16
    }
}

// impl DataRepresentationTemplate<f64> for CCSDSDataRepresentationTemplate {
//     fn compression_type(&self) -> String {
//         "CCSDS".into()
//     }

// 	fn bit_count_per_datapoint(&self) -> usize {
// 		self.bit_count() as usize
//     }

// 	fn unpack(&self, bits: &BitSlice<u8, Msb0>) -> Result<Vec<f64>, GribberishError> {
//         let bytes = bits.to_bitvec().into();

//         println!("what is going on (v3)? {:?} {:?} {:?}", self.block_size(), self.ccsds_compression_options_mask(), self.reference_sample_interval());

//         let output_value: Vec<f64> = extract_ccsds_data(bytes, self.block_size(), self.ccsds_compression_options_mask(), 
//         // TEMP
//         1038240 * 8,
//         self.reference_sample_interval())?
//             .into_iter()
//             .scale_value_by(self.binary_scale_factor(), self.decimal_scale_factor(), self.reference_value())
//             .collect();

//         Ok(output_value)
// 	}
// }


fn read_f64_from_bytes(bytes: &[u8]) -> Vec<f64> {
    // Ensure we're reading complete f64s (8 bytes each)
    let mut result = Vec::with_capacity(bytes.len() / 8);

    // Process chunks of 8 bytes
    for chunk in bytes.chunks(8) {
        if chunk.len() == 8 {
            // Only process complete f64s
            let value = f64::from_be_bytes(chunk.try_into().unwrap());
            result.push(value);
        }
    }

    result
}

impl DataRepresentationTemplate<f64> for CCSDSDataRepresentationTemplate {
    fn compression_type(&self) -> String {
        "CCSDS".into()
    }

	fn bit_count_per_datapoint(&self) -> usize {
		self.bit_count() as usize
    }

	fn unpack(&self, bits: &BitSlice<u8, Msb0>) -> Result<Vec<f64>, GribberishError> {
        let bytes: Vec<u8> = bits.to_bitvec().into();

        // println!("what is going on (v4)? {:?} {:?} {:?}", self.block_size(), self.ccsds_compression_options_mask(), self.reference_sample_interval());

        let outputwr = extract_ccsds_data(bytes, self.block_size(), self.ccsds_compression_options_mask(), 
        // TEMP
        1038240 * 8,
        self.reference_sample_interval());

        match outputwr {
            Ok(output_value) => {
                Ok(output_value.into_iter()
                    .scale_value_by(self.binary_scale_factor(), self.decimal_scale_factor(), self.reference_value())
                    .collect())
            }
            Err(e) => Err(e),
        }

        // TESTING
        // let output_value = read_f64_from_bytes(bytes.as_slice());

        // Ok(output_value)
	}
}






