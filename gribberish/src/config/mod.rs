//! Configuration system for gribberish
//!
//! This module provides a configuration system that allows users to:
//! - Define custom GRIB parameters via YAML/JSON files
//! - Extend the library with custom template definitions
//! - Configure backend preferences (native vs eccodes)
//! - Define field transformations
//!
//! # Example
//!
//! ```yaml
//! version: "1.0"
//!
//! parameters:
//!   - discipline: 0
//!     category: 192
//!     number: 1
//!     name: "Custom Temperature"
//!     abbreviation: "CTEMP"
//!     unit: "K"
//!
//! backend:
//!   preferred: "eccodes"
//!   fallback: true
//! ```
//!
//! Load and use the configuration:
//!
//! ```no_run
//! use gribberish::config::{ConfigLoader, init_registry};
//!
//! // Load configuration
//! let config = ConfigLoader::from_yaml_file("gribberish-config.yaml").unwrap();
//!
//! // Initialize the global registry
//! init_registry(&config);
//!
//! // Now custom parameters are available throughout the library
//! ```

pub mod loader;
pub mod registry;
pub mod schema;

pub use loader::ConfigLoader;
pub use registry::{
    clear_registry, get_registry, init_registry, lookup_parameter, ParameterRegistry,
};
pub use schema::{
    BackendConfig, FieldConfig, GribConfig, OperationConfig, ParameterConfig,
    TransformationConfig,
};
