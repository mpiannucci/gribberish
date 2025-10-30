//! Safe Rust bindings for ECMWF eccodes library
//!
//! This crate provides safe, idiomatic Rust wrappers around the eccodes C library
//! for reading and working with GRIB files.
//!
//! # Example
//!
//! ```no_run
//! use gribberish_eccodes::{GribHandle, GribMessageIterator};
//!
//! let data = std::fs::read("example.grib2").unwrap();
//! let mut iter = GribMessageIterator::new(&data);
//!
//! while let Some(result) = iter.next() {
//!     match result {
//!         Ok((handle, offset)) => {
//!             let short_name = handle.get_string("shortName").unwrap();
//!             let values = handle.get_data().unwrap();
//!             println!("Variable: {}, {} values", short_name, values.len());
//!         }
//!         Err(e) => eprintln!("Error: {}", e),
//!     }
//! }
//! ```

pub mod error;
pub mod handle;
pub mod iterator;

pub use error::{EccodesError, Result};
pub use handle::GribHandle;
pub use iterator::GribMessageIterator;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eccodes_available() {
        // This test just verifies that eccodes is linkable
        // Actual functionality tests require GRIB data files
        assert!(true);
    }
}
