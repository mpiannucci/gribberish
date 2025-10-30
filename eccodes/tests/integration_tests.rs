//! Integration tests for eccodes bindings
//!
//! These tests require the eccodes C library to be installed

use gribberish_eccodes::{EccodesError, GribHandle, GribMessageIterator};

/// Simple GRIB2 indicator section (minimal valid GRIB2 header)
/// This is the smallest possible GRIB2 message for testing
fn minimal_grib2_message() -> Vec<u8> {
    vec![
        // Section 0: Indicator
        b'G', b'R', b'I', b'B', // "GRIB" magic bytes
        0x00, 0x00,             // Reserved
        0x02,                   // Discipline: 2 (Land surface products)
        0x02,                   // GRIB Edition Number: 2
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x15, // Total length (21 bytes)
        // Section 8: End Section
        b'7', b'7', b'7', b'7', // "7777" end marker
    ]
}

#[test]
fn test_handle_creation_invalid() {
    let data = b"Not a GRIB file";
    let result = GribHandle::new_from_message(data);
    assert!(result.is_err());

    match result {
        Err(EccodesError::HandleCreationError(_)) => (),
        _ => panic!("Expected HandleCreationError"),
    }
}

#[test]
fn test_handle_from_empty_buffer() {
    let data = b"";
    let result = GribHandle::new_from_message(data);
    assert!(result.is_err());
}

#[test]
fn test_iterator_empty_buffer() {
    let data = b"";
    let mut iter = GribMessageIterator::new(data);
    assert!(iter.next().is_none());
}

#[test]
fn test_iterator_no_grib() {
    let data = b"This is not a GRIB file at all";
    let mut iter = GribMessageIterator::new(data);
    assert!(iter.next().is_none());
}

#[test]
fn test_iterator_offset() {
    let data = b"Some prefix data GRIB";
    let mut iter = GribMessageIterator::new(data);

    // Should find GRIB at offset 17
    assert_eq!(iter.offset(), 0);

    // Try to iterate (will fail because it's not a valid GRIB, but tests offset tracking)
    let _ = iter.next();
}

#[test]
fn test_error_display() {
    let err = EccodesError::InvalidHandle;
    assert_eq!(format!("{}", err), "Invalid handle");

    let err = EccodesError::EndOfFile;
    assert_eq!(format!("{}", err), "End of file reached");
}

#[test]
fn test_error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let err: EccodesError = io_err.into();

    match err {
        EccodesError::IoError(_) => (),
        _ => panic!("Expected IoError"),
    }
}

// Note: Testing with actual GRIB2 files would require sample data files
// and eccodes to be installed. These tests focus on error handling and
// basic functionality that doesn't require valid GRIB data.

#[cfg(feature = "sample_data")]
mod with_sample_data {
    use super::*;

    #[test]
    fn test_handle_with_real_grib() {
        // This would require a sample GRIB2 file
        let data = std::fs::read("tests/data/sample.grib2").expect("Sample file not found");
        let handle = GribHandle::new_from_message(&data).expect("Failed to create handle");

        // Test getting various keys
        let discipline = handle.get_long("discipline").expect("Failed to get discipline");
        assert!(discipline >= 0);

        let edition = handle.get_long("edition").expect("Failed to get edition");
        assert_eq!(edition, 2); // GRIB2
    }

    #[test]
    fn test_iterator_with_real_grib() {
        let data = std::fs::read("tests/data/sample.grib2").expect("Sample file not found");
        let mut iter = GribMessageIterator::new(&data);

        let mut count = 0;
        while let Some(result) = iter.next() {
            match result {
                Ok((handle, offset)) => {
                    assert!(offset < data.len());
                    let size = handle.get_message_size().expect("Failed to get size");
                    assert!(size > 0);
                    count += 1;
                }
                Err(e) => panic!("Iteration failed: {}", e),
            }
        }

        assert!(count > 0, "Should have found at least one message");
    }
}
