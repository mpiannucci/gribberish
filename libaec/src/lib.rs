//! Minimal Rust wrapper for libaec (Adaptive Entropy Coding library)
//!
//! This crate provides a simple interface to the libaec library for CCSDS
//! lossless compression.

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
        
        // Resize output to actual decoded size
        let decoded_size = output.len() - self.stream.avail_out;
        output.truncate(decoded_size);
        
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
}
