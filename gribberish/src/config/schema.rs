//! Configuration schema definitions
//!
//! This module defines the structure of configuration files that can be used
//! to extend gribberish with custom parameters and templates.

use serde::{Deserialize, Serialize};

/// Top-level configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GribConfig {
    /// Configuration schema version
    #[serde(default = "default_version")]
    pub version: String,

    /// Custom parameter definitions
    #[serde(default)]
    pub parameters: Vec<ParameterConfig>,

    /// Custom template definitions
    #[serde(default)]
    pub templates: TemplateCollectionConfig,

    /// Field transformations
    #[serde(default)]
    pub transformations: Vec<TransformationConfig>,

    /// Backend preferences
    #[serde(default)]
    pub backend: BackendConfig,
}

fn default_version() -> String {
    "1.0".to_string()
}

/// Custom parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParameterConfig {
    /// Discipline code (0-255)
    pub discipline: u8,

    /// Parameter category (0-255)
    pub category: u8,

    /// Parameter number (0-255)
    pub number: u8,

    /// Full name of the parameter
    pub name: String,

    /// Short abbreviation
    pub abbreviation: String,

    /// Unit of measurement
    pub unit: String,

    /// Optional description
    #[serde(default)]
    pub description: Option<String>,
}

/// Collection of custom templates
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateCollectionConfig {
    /// Product definition templates
    #[serde(default)]
    pub product: Vec<ProductTemplateConfig>,

    /// Grid definition templates
    #[serde(default)]
    pub grid_definition: Vec<GridTemplateConfig>,

    /// Data representation templates
    #[serde(default)]
    pub data_representation: Vec<DataTemplateConfig>,
}

/// Product template definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductTemplateConfig {
    /// Template number
    pub number: u16,

    /// Template name
    pub name: String,

    /// Field definitions
    pub fields: Vec<FieldConfig>,

    /// Optional description
    #[serde(default)]
    pub description: Option<String>,
}

/// Grid definition template
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridTemplateConfig {
    /// Template number
    pub number: u16,

    /// Template name
    pub name: String,

    /// Field definitions
    pub fields: Vec<FieldConfig>,

    /// Optional projection string
    #[serde(default)]
    pub projection: Option<String>,

    /// Optional description
    #[serde(default)]
    pub description: Option<String>,
}

/// Data representation template
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataTemplateConfig {
    /// Template number
    pub number: u16,

    /// Template name
    pub name: String,

    /// Compression type
    pub compression_type: String,

    /// Field definitions
    pub fields: Vec<FieldConfig>,

    /// Optional description
    #[serde(default)]
    pub description: Option<String>,
}

/// Field definition within a template
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldConfig {
    /// Field name
    pub name: String,

    /// Byte offset from start of template
    pub offset: usize,

    /// Field length in bytes
    pub length: usize,

    /// Field type (u8, u16, u32, i8, i16, i32, f32, f64)
    #[serde(default = "default_field_type")]
    pub field_type: String,

    /// Optional description
    #[serde(default)]
    pub description: Option<String>,
}

fn default_field_type() -> String {
    "u8".to_string()
}

/// Field transformation definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformationConfig {
    /// Variable to apply transformation to (abbreviation)
    pub variable: String,

    /// Ordered list of operations to apply
    pub operations: Vec<OperationConfig>,
}

/// Transformation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum OperationConfig {
    /// Scale values by a factor and add an offset
    Scale {
        factor: f64,
        #[serde(default)]
        offset: f64,
    },

    /// Convert units
    UnitConversion { from: String, to: String },

    /// Apply a custom formula
    Formula { expression: String },

    /// Clip values to a range
    Clip { min: Option<f64>, max: Option<f64> },
}

/// Backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackendConfig {
    /// Preferred backend ("native", "eccodes", or "auto")
    #[serde(default = "default_backend")]
    pub preferred: String,

    /// Fallback to native if preferred backend fails
    #[serde(default = "default_fallback")]
    pub fallback: bool,
}

impl Default for BackendConfig {
    fn default() -> Self {
        BackendConfig {
            preferred: default_backend(),
            fallback: default_fallback(),
        }
    }
}

fn default_backend() -> String {
    "auto".to_string()
}

fn default_fallback() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_config_deserialization() {
        let yaml = r#"
discipline: 0
category: 192
number: 1
name: "Custom Temperature"
abbreviation: "CTEMP"
unit: "K"
        "#;

        let config: ParameterConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.discipline, 0);
        assert_eq!(config.category, 192);
        assert_eq!(config.abbreviation, "CTEMP");
    }

    #[test]
    fn test_full_config_deserialization() {
        let yaml = r#"
version: "1.0"
parameters:
  - discipline: 0
    category: 192
    number: 1
    name: "Custom Temperature"
    abbreviation: "CTEMP"
    unit: "K"
backend:
  preferred: "eccodes"
  fallback: true
        "#;

        let config: GribConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.version, "1.0");
        assert_eq!(config.parameters.len(), 1);
        assert_eq!(config.backend.preferred, "eccodes");
    }
}
