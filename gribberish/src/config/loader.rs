//! Configuration file loader
//!
//! This module provides functionality to load and merge configuration files
//! from various sources (YAML, JSON).

use crate::config::schema::GribConfig;
use crate::error::GribberishError;
use std::fs;
use std::path::Path;

/// Configuration loader
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load configuration from a YAML file
    ///
    /// # Arguments
    /// * `path` - Path to the YAML configuration file
    ///
    /// # Returns
    /// The loaded configuration or an error
    pub fn from_yaml_file<P: AsRef<Path>>(path: P) -> Result<GribConfig, GribberishError> {
        let content = fs::read_to_string(path.as_ref()).map_err(|e| {
            GribberishError::MessageError(format!("Failed to read config file: {}", e))
        })?;

        Self::from_yaml_str(&content)
    }

    /// Load configuration from a YAML string
    ///
    /// # Arguments
    /// * `content` - YAML content as a string
    ///
    /// # Returns
    /// The loaded configuration or an error
    pub fn from_yaml_str(content: &str) -> Result<GribConfig, GribberishError> {
        serde_yaml::from_str(content).map_err(|e| {
            GribberishError::MessageError(format!("Failed to parse YAML config: {}", e))
        })
    }

    /// Load configuration from a JSON file
    ///
    /// # Arguments
    /// * `path` - Path to the JSON configuration file
    ///
    /// # Returns
    /// The loaded configuration or an error
    pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<GribConfig, GribberishError> {
        let content = fs::read_to_string(path.as_ref()).map_err(|e| {
            GribberishError::MessageError(format!("Failed to read config file: {}", e))
        })?;

        Self::from_json_str(&content)
    }

    /// Load configuration from a JSON string
    ///
    /// # Arguments
    /// * `content` - JSON content as a string
    ///
    /// # Returns
    /// The loaded configuration or an error
    pub fn from_json_str(content: &str) -> Result<GribConfig, GribberishError> {
        serde_json::from_str(content).map_err(|e| {
            GribberishError::MessageError(format!("Failed to parse JSON config: {}", e))
        })
    }

    /// Load configuration from a file, auto-detecting the format
    ///
    /// The format is determined by the file extension (.yaml, .yml, or .json)
    ///
    /// # Arguments
    /// * `path` - Path to the configuration file
    ///
    /// # Returns
    /// The loaded configuration or an error
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<GribConfig, GribberishError> {
        let path_ref = path.as_ref();
        let extension = path_ref
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        match extension.to_lowercase().as_str() {
            "yaml" | "yml" => Self::from_yaml_file(path_ref),
            "json" => Self::from_json_file(path_ref),
            _ => Err(GribberishError::MessageError(format!(
                "Unsupported config file format: {}",
                extension
            ))),
        }
    }

    /// Merge multiple configurations together
    ///
    /// Configurations are merged in order, with later configs taking precedence.
    /// Parameters and templates from all configs are combined.
    ///
    /// # Arguments
    /// * `configs` - Vector of configurations to merge
    ///
    /// # Returns
    /// A single merged configuration
    pub fn merge(configs: Vec<GribConfig>) -> GribConfig {
        let mut merged = GribConfig {
            version: "1.0".to_string(),
            parameters: Vec::new(),
            templates: Default::default(),
            transformations: Vec::new(),
            backend: Default::default(),
        };

        for config in configs {
            // Merge parameters
            merged.parameters.extend(config.parameters);

            // Merge templates
            merged
                .templates
                .product
                .extend(config.templates.product);
            merged
                .templates
                .grid_definition
                .extend(config.templates.grid_definition);
            merged
                .templates
                .data_representation
                .extend(config.templates.data_representation);

            // Merge transformations
            merged.transformations.extend(config.transformations);

            // Use the last backend config
            merged.backend = config.backend;
            merged.version = config.version;
        }

        merged
    }

    /// Load configurations from multiple files and merge them
    ///
    /// # Arguments
    /// * `paths` - Vector of paths to configuration files
    ///
    /// # Returns
    /// A single merged configuration or an error
    pub fn from_files<P: AsRef<Path>>(paths: Vec<P>) -> Result<GribConfig, GribberishError> {
        let mut configs = Vec::new();

        for path in paths {
            let config = Self::from_file(path)?;
            configs.push(config);
        }

        Ok(Self::merge(configs))
    }

    /// Search for and load configuration files from standard locations
    ///
    /// Searches in the following order:
    /// 1. Current directory: ./gribberish-config.yaml
    /// 2. Home directory: ~/.gribberish/config.yaml
    /// 3. System directory: /etc/gribberish/config.yaml
    ///
    /// All found configs are merged together with later configs taking precedence.
    ///
    /// # Returns
    /// A merged configuration from all found files, or a default config if none found
    pub fn from_standard_locations() -> GribConfig {
        let mut configs = Vec::new();

        // System config
        #[cfg(unix)]
        if let Ok(config) = Self::from_file("/etc/gribberish/config.yaml") {
            configs.push(config);
        }

        // User config
        if let Some(home) = std::env::var_os("HOME") {
            let user_config = Path::new(&home).join(".gribberish/config.yaml");
            if let Ok(config) = Self::from_file(user_config) {
                configs.push(config);
            }
        }

        // Project config
        if let Ok(config) = Self::from_file("./gribberish-config.yaml") {
            configs.push(config);
        }

        if configs.is_empty() {
            // Return default config
            GribConfig {
                version: "1.0".to_string(),
                parameters: Vec::new(),
                templates: Default::default(),
                transformations: Vec::new(),
                backend: Default::default(),
            }
        } else {
            Self::merge(configs)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_yaml_str() {
        let yaml = r#"
version: "1.0"
parameters:
  - discipline: 0
    category: 192
    number: 1
    name: "Test Param"
    abbreviation: "TEST"
    unit: "K"
        "#;

        let config = ConfigLoader::from_yaml_str(yaml).unwrap();
        assert_eq!(config.parameters.len(), 1);
        assert_eq!(config.parameters[0].abbreviation, "TEST");
    }

    #[test]
    fn test_from_json_str() {
        let json = r#"
{
  "version": "1.0",
  "parameters": [
    {
      "discipline": 0,
      "category": 192,
      "number": 1,
      "name": "Test Param",
      "abbreviation": "TEST",
      "unit": "K"
    }
  ]
}
        "#;

        let config = ConfigLoader::from_json_str(json).unwrap();
        assert_eq!(config.parameters.len(), 1);
        assert_eq!(config.parameters[0].abbreviation, "TEST");
    }

    #[test]
    fn test_merge_configs() {
        let config1 = GribConfig {
            version: "1.0".to_string(),
            parameters: vec![],
            templates: Default::default(),
            transformations: vec![],
            backend: Default::default(),
        };

        let mut config2 = config1.clone();
        config2.backend.backend = "native".to_string();

        let merged = ConfigLoader::merge(vec![config1, config2]);
        assert_eq!(merged.backend.backend, "native");
    }
}
