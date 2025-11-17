/// GRIB1 format support
///
/// This module provides native Rust parsing for GRIB Edition 1 files.
/// GRIB1 is the legacy format still used by some data providers (e.g., ECMWF).
///
/// GRIB1 Message Structure:
/// - Section 0: Indicator Section (8 bytes)
/// - Section 1: Product Definition Section (PDS)
/// - Section 2: Grid Description Section (GDS) - optional
/// - Section 3: Bitmap Section (BMS) - optional
/// - Section 4: Binary Data Section (BDS)
/// - Section 5: End Section ('7777')

pub mod indicator;
pub mod product_definition;
pub mod grid_description;
pub mod bitmap;
pub mod binary_data;
pub mod message;
pub mod parameters;

pub use message::Grib1Message;
