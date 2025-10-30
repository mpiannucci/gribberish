//! Native Rust backend implementation
//!
//! This backend uses the existing pure-Rust GRIB2 parsing implementation.

use crate::backends::GribBackend;
use crate::error::GribberishError;
use crate::message::{Message, MessageIterator};
use crate::message_metadata::MessageMetadata;

/// Native Rust GRIB parsing backend
///
/// This backend uses the existing pure-Rust implementation that parses
/// GRIB2 files from scratch without external dependencies.
pub struct NativeBackend;

impl GribBackend for NativeBackend {
    fn parse_message(
        &self,
        data: &[u8],
        offset: usize,
    ) -> Result<(MessageMetadata, Vec<f64>, usize), GribberishError> {
        let message = Message::from_data(data, offset)
            .ok_or_else(|| GribberishError::MessageError("Failed to parse message".to_string()))?;

        let metadata = MessageMetadata::try_from(&message)?;
        let values = message.data()?;
        let length = message.len();

        Ok((metadata, values, length))
    }

    fn scan_messages(&self, data: &[u8]) -> Vec<(usize, usize)> {
        let mut results = Vec::new();
        let mut iter = MessageIterator::from_data(data, 0);

        while let Some(message) = iter.next() {
            results.push((message.byte_offset(), message.len()));
        }

        results
    }

    fn name(&self) -> &str {
        "native"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_native_backend_name() {
        let backend = NativeBackend;
        assert_eq!(backend.name(), "native");
    }

    #[test]
    fn test_native_backend_scan_empty() {
        let backend = NativeBackend;
        let data = b"";
        let messages = backend.scan_messages(data);
        assert_eq!(messages.len(), 0);
    }
}
