//! High-level API with backend selection support
//!
//! This module provides an enhanced API that allows selecting different
//! GRIB parsing backends (native Rust or eccodes).

use crate::backends::{default_backend, get_backend, BackendType, GribBackend};
use crate::data_message::DataMessage;
use crate::error::GribberishError;
use std::collections::HashMap;

/// Read and parse all GRIB messages from data using the specified backend
///
/// # Arguments
/// * `data` - The GRIB file data as a byte slice
/// * `backend` - The backend to use for parsing
///
/// # Returns
/// A vector of DataMessage structs containing metadata and decoded values
///
/// # Example
/// ```no_run
/// use gribberish::api::read_all_messages;
/// use gribberish::backends::BackendType;
///
/// let data = std::fs::read("example.grib2").unwrap();
/// let messages = read_all_messages(&data, BackendType::Native).unwrap();
///
/// for msg in messages {
///     println!("Variable: {}, {} values", msg.metadata.var, msg.data.len());
/// }
/// ```
pub fn read_all_messages(
    data: &[u8],
    backend_type: BackendType,
) -> Result<Vec<DataMessage>, GribberishError> {
    let backend = get_backend(backend_type);
    read_all_messages_with_backend(data, backend.as_ref())
}

/// Read and parse all GRIB messages using a specific backend instance
///
/// # Arguments
/// * `data` - The GRIB file data
/// * `backend` - Reference to a backend implementation
///
/// # Returns
/// A vector of DataMessage structs
pub fn read_all_messages_with_backend(
    data: &[u8],
    backend: &dyn GribBackend,
) -> Result<Vec<DataMessage>, GribberishError> {
    let mut messages = Vec::new();
    let scanned = backend.scan_messages(data);

    for (offset, _size) in scanned {
        let (metadata, values, _) = backend.parse_message(data, offset)?;
        messages.push(DataMessage {
            metadata,
            data: values,
        });
    }

    Ok(messages)
}

/// Read and parse all GRIB messages using the default backend
///
/// The default backend is selected based on available features:
/// - If 'eccodes' feature is enabled, uses eccodes backend
/// - Otherwise, uses native Rust backend
///
/// # Arguments
/// * `data` - The GRIB file data
///
/// # Returns
/// A vector of DataMessage structs
pub fn read_all_messages_default(data: &[u8]) -> Result<Vec<DataMessage>, GribberishError> {
    let backend = default_backend();
    read_all_messages_with_backend(data, backend.as_ref())
}

/// Scan for message offsets using the specified backend
///
/// This is faster than full parsing if you only need to know where
/// messages are located in the file.
///
/// # Arguments
/// * `data` - The GRIB file data
/// * `backend_type` - The backend to use
///
/// # Returns
/// A vector of (offset, size) tuples for each message
pub fn scan_messages_with_backend(
    data: &[u8],
    backend_type: BackendType,
) -> Vec<(usize, usize)> {
    let backend = get_backend(backend_type);
    backend.scan_messages(data)
}

/// Parse a single message at a specific offset
///
/// # Arguments
/// * `data` - The GRIB file data
/// * `offset` - The byte offset where the message starts
/// * `backend_type` - The backend to use
///
/// # Returns
/// A DataMessage or an error
pub fn read_message_at_offset(
    data: &[u8],
    offset: usize,
    backend_type: BackendType,
) -> Result<DataMessage, GribberishError> {
    let backend = get_backend(backend_type);
    let (metadata, values, _) = backend.parse_message(data, offset)?;
    Ok(DataMessage {
        metadata,
        data: values,
    })
}

/// Build an index of messages grouped by variable
///
/// # Arguments
/// * `data` - The GRIB file data
/// * `backend_type` - The backend to use
///
/// # Returns
/// A HashMap mapping variable keys to (index, offset) tuples
pub fn build_message_index(
    data: &[u8],
    backend_type: BackendType,
) -> Result<HashMap<String, Vec<(usize, usize)>>, GribberishError> {
    let backend = get_backend(backend_type);
    let scanned = backend.scan_messages(data);
    let mut index: HashMap<String, Vec<(usize, usize)>> = HashMap::new();

    for (idx, (offset, _size)) in scanned.iter().enumerate() {
        let (metadata, _, _) = backend.parse_message(data, *offset)?;
        index
            .entry(metadata.var.clone())
            .or_insert_with(Vec::new)
            .push((idx, *offset));
    }

    Ok(index)
}

/// Filter messages by variable name
///
/// # Arguments
/// * `data` - The GRIB file data
/// * `variable` - The variable abbreviation to filter for (e.g., "TMP", "UGRD")
/// * `backend_type` - The backend to use
///
/// # Returns
/// A vector of DataMessage structs for the specified variable
pub fn filter_messages_by_variable(
    data: &[u8],
    variable: &str,
    backend_type: BackendType,
) -> Result<Vec<DataMessage>, GribberishError> {
    let backend = get_backend(backend_type);
    let scanned = backend.scan_messages(data);
    let mut messages = Vec::new();

    for (offset, _size) in scanned {
        let (metadata, values, _) = backend.parse_message(data, offset)?;
        if metadata.var == variable {
            messages.push(DataMessage {
                metadata,
                data: values,
            });
        }
    }

    Ok(messages)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_messages_native() {
        let data = b"";
        let messages = scan_messages_with_backend(data, BackendType::Native);
        assert_eq!(messages.len(), 0);
    }
}
