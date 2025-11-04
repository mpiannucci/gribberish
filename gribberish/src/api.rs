//! High-level API for GRIB parsing
//!
//! This module provides a simplified API for parsing GRIB files.
//! Supports both GRIB1 and GRIB2 formats using pure Rust implementations.

use crate::data_message::DataMessage;
use crate::error::GribberishError;
use crate::parser;
use std::collections::HashMap;

/// Read and parse all GRIB messages from data
///
/// # Arguments
/// * `data` - The GRIB file data as a byte slice
///
/// # Returns
/// A vector of DataMessage structs containing metadata and decoded values
///
/// # Example
/// ```no_run
/// use gribberish::api::read_all_messages;
///
/// let data = std::fs::read("example.grib2").unwrap();
/// let messages = read_all_messages(&data).unwrap();
///
/// for msg in messages {
///     println!("Variable: {}, {} values", msg.metadata.var, msg.data.len());
/// }
/// ```
pub fn read_all_messages(data: &[u8]) -> Result<Vec<DataMessage>, GribberishError> {
    let mut messages = Vec::new();
    let scanned = parser::scan_messages(data);

    for (offset, _size) in scanned {
        let (metadata, values, _) = parser::parse_message(data, offset)?;
        messages.push(DataMessage {
            metadata,
            data: values,
        });
    }

    Ok(messages)
}

/// Scan for message offsets
///
/// This is faster than full parsing if you only need to know where
/// messages are located in the file.
///
/// # Arguments
/// * `data` - The GRIB file data
///
/// # Returns
/// A vector of (offset, size) tuples for each message
pub fn scan_messages(data: &[u8]) -> Vec<(usize, usize)> {
    parser::scan_messages(data)
}

/// Parse a single message at a specific offset
///
/// # Arguments
/// * `data` - The GRIB file data
/// * `offset` - The byte offset where the message starts
///
/// # Returns
/// A DataMessage or an error
pub fn read_message_at_offset(
    data: &[u8],
    offset: usize,
) -> Result<DataMessage, GribberishError> {
    let (metadata, values, _) = parser::parse_message(data, offset)?;
    Ok(DataMessage {
        metadata,
        data: values,
    })
}

/// Build an index of messages grouped by variable
///
/// # Arguments
/// * `data` - The GRIB file data
///
/// # Returns
/// A HashMap mapping variable keys to (index, offset) tuples
pub fn build_message_index(
    data: &[u8],
) -> Result<HashMap<String, Vec<(usize, usize)>>, GribberishError> {
    let scanned = parser::scan_messages(data);
    let mut index: HashMap<String, Vec<(usize, usize)>> = HashMap::new();

    for (idx, (offset, _size)) in scanned.iter().enumerate() {
        let (metadata, _, _) = parser::parse_message(data, *offset)?;
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
///
/// # Returns
/// A vector of DataMessage structs for the specified variable
pub fn filter_messages_by_variable(
    data: &[u8],
    variable: &str,
) -> Result<Vec<DataMessage>, GribberishError> {
    let scanned = parser::scan_messages(data);
    let mut messages = Vec::new();

    for (offset, _size) in scanned {
        let (metadata, values, _) = parser::parse_message(data, offset)?;
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
    fn test_scan_messages_empty() {
        let data = b"";
        let messages = scan_messages(data);
        assert_eq!(messages.len(), 0);
    }
}
