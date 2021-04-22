use crate::templates::template::{Template, TemplateType};
use super::data_representation_template::DataRepresentationTemplate;
use super::tables::{CompressionType, OriginalFieldValue};
use crate::unwrap_or_return;
use crate::utils::{from_bits, read_f32_from_bytes, read_i16_from_bytes, bits_to_bytes};
use num::Float;
use std::ops::Range;
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
    
    // fn decode_bits(&self, bits: Vec<u8>) -> Result<Vec<u8>, String> {
        // TODO: MATCH THIS WORKFLOW https://github.com/ecmwf/eccodes/blob/develop/src/grib_openjpeg_encoding.c
        // TODO: https://github.com/uclouvain/openjpeg/blob/master/src/lib/openjp2/openjpeg.h
        // TODO: https://docs.rs/openjpeg-sys/1.0.1/openjpeg_sys/

        // let reader = BufReader::new(bits.as_slice());
        // let mut decoder = jpeg_decoder::Decoder::new(reader);

        // let bytes = bits_to_bytes(bits).unwrap();

        // println!("{:?}", bytes);

        // let decoder = openjpeg_sys::opj_create_decompress(openjpeg_sys::OPJ_CODEC_FORMAT::OPJ_CODEC_J2K);

        // let stream = openjpeg_sys::opj_stream_create

        // let image = match jpeg2000::decode::from_memory(
        //     bytes.as_slice(), 
        //     jpeg2000::decode::Codec::J2K, 
        //     jpeg2000::decode::DecodeConfig {
        //         default_colorspace: Some(jpeg2000::decode::ColorSpace::GRAY),
        //         discard_level: 0,
        //     }, 
        //     None,
        // ) {
        //     Ok(i) => Ok(i),
        //     Err(e) => Err(String::from(format!("Error decoding JPEG codestream {}", e)))
        // }?;

        // let mask = (1 << self.bit_count_per_datapoint()) - 1;

        // let bytes = image.raw_pixels().iter().map(|p| {
        //     p & mask
        // }).collect();

        // Ok(bytes)

        // Err(String::from("akljsjasd"))

        // match decoder.decode() {
        //     Ok(decoded_bits) => Ok(decoded_bits),
        //     Err(e) => Err(format!("Error decoding JPEG codestream {}", e).into()),
        // }
    // }

    fn scaled_value(&self, raw_value: f64) -> f64 {
        let reference_value: f64 = self.reference_value().into();
        let binary_scale_factor: i32 = self.binary_scale_factor().into();
        let decimal_scale_factor: i32 = self.decimal_scale_factor().into();

        (reference_value + (raw_value * 2.0.powi(binary_scale_factor)))
            / 10.0.powi(decimal_scale_factor)
    }
	
	fn unpack_range(&self, bits: Vec<u8>, range: Range<usize>) -> Result<Vec<f64>, &'static str> {

        let bytes = bits_to_bytes(bits).unwrap();

        unsafe {
            let parameters = &mut openjpeg_sys::opj_dparameters{
                cp_reduce: 0,
                cp_layer: 0,
                infile: [0; 4096],
                outfile: [0; 4096],
                decod_format: 0,
                cod_format: 0,
                DA_x0: 0,
                DA_x1: 0,
                DA_y0: 0,
                DA_y1: 0,
                m_verbose: 0,
                tile_index: 0,
                nb_tile_to_decode: 0,
                jpwl_correct: 0,
                jpwl_exp_comps: 0,
                jpwl_max_tiles: 0,
                flags: 0,
            } as *mut openjpeg_sys::opj_dparameters_t;
            openjpeg_sys::opj_set_default_decoder_parameters(parameters);
            let dinfo = openjpeg_sys::opj_create_decompress(openjpeg_sys::CODEC_FORMAT::OPJ_CODEC_J2K);
            openjpeg_sys::opj_setup_decoder(dinfo, parameters);

            // TODO: Actually decode 
        }

        let mut v = Vec::new();

        let bits_per_val: usize = self.bit_count().into();
        if bits_per_val == 0 {
            return Err("Invalid bits per value size of 0");
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
                "failed to convert value to u32"
            )
            .into();
            let val = self.scaled_value(raw_value);
            v.push(val);
        }

        Ok(v)
	}

    fn unpack_all(&self, bits: Vec<u8>) -> Result<Vec<f64>, &'static str> {
		let bit_count = bits.len();
		self.unpack_range(bits, 0..bit_count)
    }
}