//! CCSDS decompression using libaec
//!
//! This module provides CCSDS lossless decompression functionality for GRIB2 files
//! using the libaec library.

use crate::error::GribberishError;
use libaec_sys::*;
use std::ffi::c_int;
use thiserror::Error;

/// Errors that can occur during decompression
#[derive(Error, Debug)]
pub enum AecError {
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Stream error: {0}")]
    Stream(String),
    #[error("Data error: {0}")]
    Data(String),
    #[error("Memory error: {0}")]
    Memory(String),
    #[error("Unknown error: {0}")]
    Unknown(c_int),
}

impl From<c_int> for AecError {
    fn from(code: c_int) -> Self {
        match code {
            AEC_CONF_ERROR => AecError::Config("Invalid configuration".to_string()),
            AEC_STREAM_ERROR => AecError::Stream("Stream processing error".to_string()),
            AEC_DATA_ERROR => AecError::Data("Invalid input data".to_string()),
            AEC_MEM_ERROR => AecError::Memory("Memory allocation error".to_string()),
            _ => AecError::Unknown(code),
        }
    }
}

/// CCSDS decoder for decompressing GRIB2 data
pub struct Decoder {
    stream: aec_stream,
}

impl Decoder {
    /// Create a new CCSDS decoder
    pub fn new(
        bits_per_sample: u32,
        block_size: u32,
        reference_sample_interval: u32,
        compression_options_mask: u8,
    ) -> Result<Self, AecError> {
        let mut stream = unsafe { std::mem::zeroed::<aec_stream>() };

        // Configure the stream
        stream.bits_per_sample = bits_per_sample;
        stream.block_size = block_size;
        stream.rsi = reference_sample_interval;

        // Parse compression flags
        let mut flags = 0u32;
        if (compression_options_mask & 0x01) != 0 {
            flags |= AEC_DATA_SIGNED;
        }
        if (compression_options_mask & 0x02) != 0 {
            flags |= AEC_DATA_PREPROCESS;
        }
        if (compression_options_mask & 0x04) != 0 {
            flags |= AEC_DATA_MSB;
        }
        if (compression_options_mask & 0x08) != 0 {
            flags |= AEC_RESTRICTED;
        }
        if (compression_options_mask & 0x10) != 0 {
            flags |= AEC_PAD_RSI;
        }

        stream.flags = flags;

        // Initialize decoder
        let result = unsafe { aec_decode_init(&mut stream) };
        if result != AEC_OK as c_int {
            return Err(AecError::from(result));
        }

        Ok(Self { stream })
    }

    /// Decode compressed data
    pub fn decode(&mut self, input: &[u8], output_size: usize) -> Result<Vec<u8>, AecError> {
        let mut output = vec![0u8; output_size];

        // Set up input/output buffers
        self.stream.next_in = input.as_ptr();
        self.stream.avail_in = input.len();
        self.stream.next_out = output.as_mut_ptr();
        self.stream.avail_out = output.len();

        // Decode
        let result = unsafe { aec_decode(&mut self.stream, AEC_FLUSH as i32) };
        if result as u32 != AEC_OK {
            return Err(AecError::from(result));
        }

        Ok(output)
    }
}

impl Drop for Decoder {
    fn drop(&mut self) {
        unsafe {
            aec_decode_end(&mut self.stream);
        }
    }
}

/// Extract CCSDS compressed data using libaec
///
/// # Arguments
/// * `data` - Compressed input data
/// * `block_len` - Block size used during compression (8-64)
/// * `compression_options_mask` - CCSDS compression options mask/flags
/// * `avail_out` - Expected size of decompressed output in bytes
/// * `reference_sample_interval` - Reference sample interval (RSI)
/// * `bits_per_sample` - Number of bits per sample (1-32)
///
/// # Returns
/// Vector of decompressed f32 values
pub fn extract_ccsds_data(
    data: Vec<u8>,
    block_len: u8,
    compression_options_mask: u8,
    avail_out: usize,
    reference_sample_interval: u16,
    bits_per_sample: usize,
) -> Result<Vec<f32>, GribberishError> {
    // Validate inputs
    if data.is_empty() {
        return Err(GribberishError::MessageError(
            "Empty input data".to_string(),
        ));
    }

    if bits_per_sample == 0 {
        return Ok(vec![]);
    }

    if bits_per_sample > 32 {
        return Err(GribberishError::MessageError(
            "bits_per_sample cannot exceed 32".to_string(),
        ));
    }

    // Parse compression flags from the options mask
    let big_endian = (compression_options_mask & 0x04) != 0;

    // Build decoder with parameters
    let mut decoder = Decoder::new(
        bits_per_sample as u32,
        block_len as u32,
        reference_sample_interval as u32,
        compression_options_mask,
    )
    .map_err(|e| GribberishError::MessageError(format!("Failed to create decoder: {}", e)))?;

    let bytes_per_sample = (bits_per_sample + 7) / 8;
    // Calculate the number of samples we expect
    let num_samples = avail_out / bytes_per_sample;

    // Validate that avail_out is correctly sized
    if avail_out % bytes_per_sample != 0 {
        return Err(GribberishError::MessageError(format!(
            "avail_out ({}) is not a multiple of bytes_per_sample ({})",
            avail_out, bytes_per_sample
        )));
    }

    // Decode the compressed data
    let decompressed_bytes = decoder.decode(&data, avail_out).map_err(|e| match e {
        AecError::Config(msg) => {
            GribberishError::MessageError(format!("Configuration error: {}", msg))
        }
        AecError::Stream(msg) => GribberishError::MessageError(format!("Stream error: {}", msg)),
        AecError::Data(msg) => GribberishError::MessageError(format!("Data error: {}", msg)),
        AecError::Memory(msg) => GribberishError::MessageError(format!("Memory error: {}", msg)),
        AecError::Unknown(code) => {
            GribberishError::MessageError(format!("Unknown error code: {}", code))
        }
    })?;

    // Convert bytes to f32 values to match pure Rust interface
    let f32_values = bytes_to_f32_values(
        &decompressed_bytes,
        bits_per_sample,
        num_samples,
        big_endian,
    )?;

    Ok(f32_values)
}

/// Convert decompressed bytes to f32 values as expected by GRIB2 processing
fn bytes_to_f32_values(
    bytes: &[u8],
    bits_per_sample: usize,
    num_samples: usize,
    big_endian: bool,
) -> Result<Vec<f32>, GribberishError> {
    let storage_size = if bits_per_sample <= 8 {
        1
    } else if bits_per_sample <= 16 {
        2
    } else {
        4
    };

    let expected_byte_count = num_samples * storage_size;
    if bytes.len() < expected_byte_count {
        return Err(GribberishError::MessageError(format!(
            "Insufficient decompressed data: got {} bytes, expected {}",
            bytes.len(),
            expected_byte_count
        )));
    }

    let mut values = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let start_idx = i * storage_size;
        let value = match storage_size {
            1 => bytes[start_idx] as u32,
            2 => {
                // Use the correct endianness based on CCSDS compression flags
                if big_endian {
                    u16::from_be_bytes([bytes[start_idx], bytes[start_idx + 1]]) as u32
                } else {
                    u16::from_le_bytes([bytes[start_idx], bytes[start_idx + 1]]) as u32
                }
            }
            4 => {
                // Use the correct endianness based on CCSDS compression flags
                if big_endian {
                    u32::from_be_bytes([
                        bytes[start_idx],
                        bytes[start_idx + 1],
                        bytes[start_idx + 2],
                        bytes[start_idx + 3],
                    ])
                } else {
                    u32::from_le_bytes([
                        bytes[start_idx],
                        bytes[start_idx + 1],
                        bytes[start_idx + 2],
                        bytes[start_idx + 3],
                    ])
                }
            }
            _ => unreachable!(),
        };

        // Mask off unused bits if needed
        let masked_value = if bits_per_sample < storage_size * 8 {
            let mask = (1u32 << bits_per_sample) - 1;
            value & mask
        } else {
            value
        };

        values.push(masked_value as f32);
    }

    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decoder_creation() {
        let decoder = Decoder::new(16, 32, 128, 0);
        assert!(decoder.is_ok());
    }

    #[test]
    fn test_invalid_parameters() {
        // bits_per_sample = 0 should fail
        let decoder = Decoder::new(0, 32, 128, 0);
        assert!(decoder.is_err());
    }

    #[test]
    fn test_extract_ccsds_data_empty_input() {
        let result = extract_ccsds_data(vec![], 16, 0, 100, 128, 8);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_ccsds_data_zero_bits() {
        let result = extract_ccsds_data(vec![1, 2, 3, 4], 16, 0, 100, 128, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![]);
    }

    #[test]
    fn test_extract_ccsds_data_invalid_bits() {
        let result = extract_ccsds_data(vec![1, 2, 3, 4], 16, 0, 100, 128, 33);
        assert!(result.is_err());
    }

    #[test]
    fn test_bytes_to_f32_values_8bit() {
        let bytes = vec![0x10, 0x20, 0x30, 0x40];
        let result = bytes_to_f32_values(&bytes, 8, 4, true).unwrap();
        assert_eq!(result, vec![16.0, 32.0, 48.0, 64.0]);
    }

    #[test]
    fn test_bytes_to_f32_values_16bit() {
        let bytes = vec![0x12, 0x34, 0x56, 0x78]; // Big-endian 0x1234, 0x5678
        let result = bytes_to_f32_values(&bytes, 16, 2, true).unwrap();
        assert_eq!(result, vec![4660.0, 22136.0]);
    }

    #[test]
    fn test_bytes_to_f32_values_32bit() {
        let bytes = vec![0x12, 0x34, 0x56, 0x78, 0x56, 0x78, 0x9A, 0xBC];
        let result = bytes_to_f32_values(&bytes, 32, 2, true).unwrap();
        assert_eq!(result, vec![305419896.0, 1450744508.0]);
    }

    #[test]
    fn test_bytes_to_f32_values_insufficient_data() {
        let bytes = vec![0x10, 0x20]; // 2 bytes, but need 4 for 2 samples of 16-bit
        let result = bytes_to_f32_values(&bytes, 16, 2, true);
        assert!(result.is_err());
    }

    #[test]
    fn test_bytes_to_f32_values_bit_masking() {
        let bytes = vec![0xFF]; // All bits set
        let result = bytes_to_f32_values(&bytes, 4, 1, true).unwrap(); // Only 4 bits should be used
        assert_eq!(result, vec![15.0]); // Should be masked to 4 bits
    }

    #[test]
    fn test_bytes_to_f32_values_little_endian() {
        let bytes = vec![0x12, 0x34, 0x56, 0x78]; // Little-endian 0x3412, 0x7856
        let result = bytes_to_f32_values(&bytes, 16, 2, false).unwrap();
        assert_eq!(result, vec![13330.0, 30806.0]);
    }

    #[test]
    fn test_extract_ccsds_data_invalid_avail_out() {
        // Test that avail_out must be a multiple of bytes_per_sample
        // With 16 bits per sample, bytes_per_sample = 2
        // avail_out = 5 is not a multiple of 2, should return validation error
        let result = extract_ccsds_data(vec![1, 2, 3, 4], 16, 0, 5, 128, 16);
        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("not a multiple of bytes_per_sample"));
    }
}
