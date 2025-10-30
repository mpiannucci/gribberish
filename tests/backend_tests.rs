//! Tests for backend abstraction
//!
//! Tests the backend trait, native backend, and backend selection

use gribberish::backends::{default_backend, get_backend, BackendType, GribBackend};

#[test]
fn test_backend_type_default() {
    let default = BackendType::default();

    // Default should be eccodes if available, otherwise native
    #[cfg(feature = "eccodes")]
    assert_eq!(default, BackendType::Eccodes);

    #[cfg(not(feature = "eccodes"))]
    assert_eq!(default, BackendType::Native);
}

#[test]
fn test_get_native_backend() {
    let backend = get_backend(BackendType::Native);
    assert_eq!(backend.name(), "native");
}

#[test]
#[cfg(feature = "eccodes")]
fn test_get_eccodes_backend() {
    let backend = get_backend(BackendType::Eccodes);
    assert_eq!(backend.name(), "eccodes");
}

#[test]
fn test_default_backend() {
    let backend = default_backend();

    // Should return a valid backend
    #[cfg(feature = "eccodes")]
    assert_eq!(backend.name(), "eccodes");

    #[cfg(not(feature = "eccodes"))]
    assert_eq!(backend.name(), "native");
}

#[test]
fn test_native_backend_scan_empty() {
    let backend = get_backend(BackendType::Native);
    let data = b"";
    let messages = backend.scan_messages(data);
    assert_eq!(messages.len(), 0);
}

#[test]
fn test_native_backend_scan_no_grib() {
    let backend = get_backend(BackendType::Native);
    let data = b"This is not a GRIB file";
    let messages = backend.scan_messages(data);
    assert_eq!(messages.len(), 0);
}

#[test]
fn test_native_backend_parse_invalid() {
    let backend = get_backend(BackendType::Native);
    let data = b"Not a GRIB message";
    let result = backend.parse_message(data, 0);
    assert!(result.is_err());
}

#[test]
fn test_backend_trait_object() {
    // Test that we can use backends through trait objects
    fn use_backend(backend: &dyn GribBackend) -> String {
        backend.name().to_string()
    }

    let backend = get_backend(BackendType::Native);
    let name = use_backend(backend.as_ref());
    assert_eq!(name, "native");
}

#[test]
fn test_multiple_backends() {
    // Test that we can have multiple backend instances
    let backend1 = get_backend(BackendType::Native);
    let backend2 = get_backend(BackendType::Native);

    assert_eq!(backend1.name(), backend2.name());
}

#[cfg(feature = "eccodes")]
#[test]
fn test_both_backends_available() {
    let native = get_backend(BackendType::Native);
    let eccodes = get_backend(BackendType::Eccodes);

    assert_eq!(native.name(), "native");
    assert_eq!(eccodes.name(), "eccodes");
}

// Mock tests with minimal GRIB-like data
mod with_mock_data {
    use super::*;

    fn create_grib_like_data() -> Vec<u8> {
        // Not a valid GRIB, but starts with "GRIB" for testing
        let mut data = vec![b'G', b'R', b'I', b'B'];
        data.extend_from_slice(&[0x00, 0x00, 0x02, 0x02]);
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10]);
        data
    }

    #[test]
    fn test_native_backend_finds_grib_marker() {
        let backend = get_backend(BackendType::Native);
        let data = create_grib_like_data();

        // Should at least try to parse it (even if it fails)
        let result = backend.parse_message(&data, 0);

        // We expect it to fail because it's not a complete valid GRIB
        // but the test is that it attempts to parse at the GRIB marker
        assert!(result.is_err());
    }
}

#[cfg(all(test, feature = "benchmark"))]
mod benchmarks {
    use super::*;
    use std::time::Instant;

    #[test]
    fn bench_backend_creation() {
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = get_backend(BackendType::Native);
        }
        let duration = start.elapsed();
        println!("Backend creation: {:?} for 1000 iterations", duration);
    }
}
