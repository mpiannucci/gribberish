//! Tests for the high-level API
//!
//! Tests API functions with different backends

use gribberish::api::{
    build_message_index, filter_messages_by_variable, read_all_messages,
    read_all_messages_default, read_message_at_offset, scan_messages_with_backend,
};
use gribberish::backends::BackendType;

#[test]
fn test_scan_messages_empty() {
    let data = b"";
    let messages = scan_messages_with_backend(data, BackendType::Native);
    assert_eq!(messages.len(), 0);
}

#[test]
fn test_scan_messages_no_grib() {
    let data = b"This is not a GRIB file";
    let messages = scan_messages_with_backend(data, BackendType::Native);
    assert_eq!(messages.len(), 0);
}

#[test]
fn test_read_all_messages_empty() {
    let data = b"";
    let result = read_all_messages(data, BackendType::Native);

    // Should succeed but return empty vector
    assert!(result.is_ok());
    let messages = result.unwrap();
    assert_eq!(messages.len(), 0);
}

#[test]
fn test_read_all_messages_invalid() {
    let data = b"Not a GRIB file";
    let result = read_all_messages(data, BackendType::Native);

    // Should succeed with empty result (no valid messages)
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[test]
fn test_read_message_at_offset_invalid() {
    let data = b"Not a GRIB message";
    let result = read_message_at_offset(data, 0, BackendType::Native);

    // Should fail for invalid data
    assert!(result.is_err());
}

#[test]
fn test_read_all_messages_default() {
    let data = b"";
    let result = read_all_messages_default(data);

    // Should work with default backend
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[test]
fn test_build_message_index_empty() {
    let data = b"";
    let result = build_message_index(data, BackendType::Native);

    // Should succeed with empty index
    assert!(result.is_ok());
    let index = result.unwrap();
    assert_eq!(index.len(), 0);
}

#[test]
fn test_filter_messages_empty() {
    let data = b"";
    let result = filter_messages_by_variable(data, "TMP", BackendType::Native);

    // Should succeed with empty result
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[test]
fn test_api_consistency_across_functions() {
    // Test that different API functions handle the same data consistently
    let data = b"";

    let scan_result = scan_messages_with_backend(data, BackendType::Native);
    let read_result = read_all_messages(data, BackendType::Native);
    let index_result = build_message_index(data, BackendType::Native);
    let filter_result = filter_messages_by_variable(data, "TMP", BackendType::Native);

    assert!(scan_result.is_empty());
    assert!(read_result.is_ok() && read_result.unwrap().is_empty());
    assert!(index_result.is_ok() && index_result.unwrap().is_empty());
    assert!(filter_result.is_ok() && filter_result.unwrap().is_empty());
}

#[cfg(feature = "eccodes")]
mod eccodes_api_tests {
    use super::*;

    #[test]
    fn test_eccodes_scan_messages() {
        let data = b"";
        let messages = scan_messages_with_backend(data, BackendType::Eccodes);
        assert_eq!(messages.len(), 0);
    }

    #[test]
    fn test_eccodes_read_all_messages() {
        let data = b"";
        let result = read_all_messages(data, BackendType::Eccodes);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_eccodes_filter_messages() {
        let data = b"";
        let result = filter_messages_by_variable(data, "TMP", BackendType::Eccodes);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }
}

// Integration tests with mock data would go here
// These would require valid GRIB2 test files

#[cfg(feature = "integration_tests")]
mod integration {
    use super::*;
    use std::fs;

    #[test]
    fn test_read_real_grib_file() {
        let data = fs::read("tests/data/sample.grib2").expect("Sample file not found");

        // Test with native backend
        let messages = read_all_messages(&data, BackendType::Native).expect("Failed to read");
        assert!(!messages.is_empty(), "Should find messages in sample file");

        // Verify data structure
        for msg in &messages {
            assert!(!msg.data.is_empty(), "Message should have data");
            assert!(!msg.metadata.var.is_empty(), "Message should have variable name");
        }
    }

    #[test]
    fn test_scan_real_grib_file() {
        let data = fs::read("tests/data/sample.grib2").expect("Sample file not found");

        let scanned = scan_messages_with_backend(&data, BackendType::Native);
        assert!(!scanned.is_empty(), "Should find messages");

        // Verify offsets are valid
        for (offset, size) in scanned {
            assert!(offset < data.len(), "Offset should be within file");
            assert!(size > 0, "Size should be positive");
            assert!(offset + size <= data.len(), "Message should fit in file");
        }
    }

    #[test]
    fn test_filter_by_variable() {
        let data = fs::read("tests/data/sample.grib2").expect("Sample file not found");

        // First, get all messages to find a variable
        let all_messages = read_all_messages(&data, BackendType::Native).expect("Failed to read");
        assert!(!all_messages.is_empty());

        // Get the first variable name
        let var_name = &all_messages[0].metadata.var;

        // Filter for that variable
        let filtered =
            filter_messages_by_variable(&data, var_name, BackendType::Native).expect("Failed");

        assert!(!filtered.is_empty(), "Should find filtered messages");

        // Verify all filtered messages have the correct variable
        for msg in filtered {
            assert_eq!(&msg.metadata.var, var_name);
        }
    }

    #[test]
    fn test_build_index() {
        let data = fs::read("tests/data/sample.grib2").expect("Sample file not found");

        let index = build_message_index(&data, BackendType::Native).expect("Failed to build index");

        assert!(!index.is_empty(), "Index should not be empty");

        // Verify index structure
        for (var, locations) in index {
            assert!(!var.is_empty(), "Variable name should not be empty");
            assert!(!locations.is_empty(), "Should have at least one location");

            for (idx, offset) in locations {
                assert!(offset < data.len(), "Offset should be valid");
            }
        }
    }

    #[test]
    fn test_read_specific_message() {
        let data = fs::read("tests/data/sample.grib2").expect("Sample file not found");

        // Scan for offsets
        let scanned = scan_messages_with_backend(&data, BackendType::Native);
        assert!(!scanned.is_empty());

        // Read the first message at its offset
        let (offset, _) = scanned[0];
        let message =
            read_message_at_offset(&data, offset, BackendType::Native).expect("Failed to read");

        assert!(!message.data.is_empty());
        assert_eq!(message.metadata.byte_offset, offset);
    }

    #[test]
    #[cfg(feature = "eccodes")]
    fn test_compare_backends() {
        let data = fs::read("tests/data/sample.grib2").expect("Sample file not found");

        // Read with both backends
        let native_messages =
            read_all_messages(&data, BackendType::Native).expect("Native failed");
        let eccodes_messages =
            read_all_messages(&data, BackendType::Eccodes).expect("Eccodes failed");

        // Should find the same number of messages
        assert_eq!(
            native_messages.len(),
            eccodes_messages.len(),
            "Both backends should find same number of messages"
        );

        // Compare data arrays (they should be very similar, allowing for small float differences)
        for (native_msg, eccodes_msg) in native_messages.iter().zip(eccodes_messages.iter()) {
            assert_eq!(
                native_msg.data.len(),
                eccodes_msg.data.len(),
                "Data arrays should have same length"
            );

            // Variables should match
            assert_eq!(native_msg.metadata.var, eccodes_msg.metadata.var);
        }
    }
}
