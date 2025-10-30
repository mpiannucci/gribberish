//! Backend abstraction for GRIB parsing
//!
//! This module provides a trait-based abstraction that allows switching between
//! different GRIB parsing implementations (native Rust or eccodes-based).

pub mod native;

#[cfg(feature = "eccodes")]
pub mod eccodes;

use crate::error::GribberishError;
use crate::message_metadata::MessageMetadata;

/// Trait for GRIB parsing backends
///
/// This trait defines the interface that all GRIB parsing backends must implement.
/// It allows the library to support multiple parsing strategies (native Rust,
/// eccodes, etc.) with a common API.
pub trait GribBackend {
    /// Parse a single GRIB message at the given offset
    ///
    /// # Arguments
    /// * `data` - The complete GRIB file data
    /// * `offset` - The byte offset where the message starts
    ///
    /// # Returns
    /// A tuple of (metadata, data_values, message_length) or an error
    fn parse_message(
        &self,
        data: &[u8],
        offset: usize,
    ) -> Result<(MessageMetadata, Vec<f64>, usize), GribberishError>;

    /// Scan for all messages in the data
    ///
    /// # Arguments
    /// * `data` - The complete GRIB file data
    ///
    /// # Returns
    /// A vector of (offset, message_length) tuples
    fn scan_messages(&self, data: &[u8]) -> Vec<(usize, usize)>;

    /// Get the name of this backend
    fn name(&self) -> &str;
}

/// Available backend types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendType {
    /// Native Rust implementation (always available)
    Native,

    /// ECMWF eccodes-based implementation (requires 'eccodes' feature)
    #[cfg(feature = "eccodes")]
    Eccodes,
}

impl Default for BackendType {
    fn default() -> Self {
        // Prefer eccodes if available, otherwise use native
        #[cfg(feature = "eccodes")]
        return BackendType::Eccodes;

        #[cfg(not(feature = "eccodes"))]
        return BackendType::Native;
    }
}

/// Get a backend instance by type
pub fn get_backend(backend_type: BackendType) -> Box<dyn GribBackend> {
    match backend_type {
        BackendType::Native => Box::new(native::NativeBackend),

        #[cfg(feature = "eccodes")]
        BackendType::Eccodes => Box::new(eccodes::EccodesBackend),
    }
}

/// Get the default backend
pub fn default_backend() -> Box<dyn GribBackend> {
    get_backend(BackendType::default())
}
