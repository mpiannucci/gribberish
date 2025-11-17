//! Runtime registry for custom parameters and templates
//!
//! This module provides a global registry where custom parameters and templates
//! can be registered and looked up at runtime.

use crate::config::schema::{GribConfig, ParameterConfig};
use std::collections::HashMap;
use std::sync::RwLock;

/// Global parameter registry
static PARAMETER_REGISTRY: RwLock<Option<ParameterRegistry>> = RwLock::new(None);

/// Registry for custom parameters
#[derive(Debug, Clone)]
pub struct ParameterRegistry {
    /// Map from (discipline, category, number) to parameter config
    parameters: HashMap<(u8, u8, u8), ParameterConfig>,
}

impl ParameterRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        ParameterRegistry {
            parameters: HashMap::new(),
        }
    }

    /// Register a parameter
    ///
    /// # Arguments
    /// * `param` - The parameter configuration to register
    pub fn register(&mut self, param: ParameterConfig) {
        let key = (param.discipline, param.category, param.number);
        self.parameters.insert(key, param);
    }

    /// Register multiple parameters
    ///
    /// # Arguments
    /// * `params` - Vector of parameter configurations to register
    pub fn register_many(&mut self, params: Vec<ParameterConfig>) {
        for param in params {
            self.register(param);
        }
    }

    /// Look up a parameter by its codes
    ///
    /// # Arguments
    /// * `discipline` - The discipline code
    /// * `category` - The category code
    /// * `number` - The parameter number
    ///
    /// # Returns
    /// The parameter configuration if found
    pub fn lookup(&self, discipline: u8, category: u8, number: u8) -> Option<&ParameterConfig> {
        self.parameters.get(&(discipline, category, number))
    }

    /// Get all registered parameters
    pub fn all_parameters(&self) -> Vec<&ParameterConfig> {
        self.parameters.values().collect()
    }

    /// Clear all registered parameters
    pub fn clear(&mut self) {
        self.parameters.clear();
    }

    /// Get the number of registered parameters
    pub fn count(&self) -> usize {
        self.parameters.len()
    }
}

impl Default for ParameterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize the global parameter registry from a configuration
///
/// # Arguments
/// * `config` - The configuration to load into the registry
///
/// # Example
/// ```no_run
/// use gribberish::config::{ConfigLoader, init_registry};
///
/// let config = ConfigLoader::from_yaml_file("config.yaml").unwrap();
/// init_registry(&config);
/// ```
pub fn init_registry(config: &GribConfig) {
    let mut registry = ParameterRegistry::new();
    registry.register_many(config.parameters.clone());

    let mut global = PARAMETER_REGISTRY.write().unwrap();
    *global = Some(registry);
}

/// Get a reference to the global parameter registry
///
/// # Returns
/// A read-locked reference to the global registry
pub fn get_registry() -> std::sync::RwLockReadGuard<'static, Option<ParameterRegistry>> {
    PARAMETER_REGISTRY.read().unwrap()
}

/// Look up a parameter in the global registry
///
/// # Arguments
/// * `discipline` - The discipline code
/// * `category` - The category code
/// * `number` - The parameter number
///
/// # Returns
/// The parameter configuration if found in the global registry
///
/// # Example
/// ```no_run
/// use gribberish::config::lookup_parameter;
///
/// if let Some(param) = lookup_parameter(0, 192, 1) {
///     println!("Found parameter: {} ({})", param.name, param.abbreviation);
/// }
/// ```
pub fn lookup_parameter(discipline: u8, category: u8, number: u8) -> Option<ParameterConfig> {
    let registry = get_registry();
    if let Some(ref reg) = *registry {
        reg.lookup(discipline, category, number).cloned()
    } else {
        None
    }
}

/// Clear the global parameter registry
pub fn clear_registry() {
    let mut global = PARAMETER_REGISTRY.write().unwrap();
    if let Some(ref mut registry) = *global {
        registry.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_registry() {
        let mut registry = ParameterRegistry::new();

        let param = ParameterConfig {
            discipline: 0,
            category: 192,
            number: 1,
            name: "Test Parameter".to_string(),
            abbreviation: "TEST".to_string(),
            unit: "K".to_string(),
            description: None,
        };

        registry.register(param.clone());

        let found = registry.lookup(0, 192, 1);
        assert!(found.is_some());
        assert_eq!(found.unwrap().abbreviation, "TEST");

        assert!(registry.lookup(0, 192, 2).is_none());
    }

    #[test]
    fn test_global_registry() {
        clear_registry();

        let config = GribConfig {
            version: "1.0".to_string(),
            parameters: vec![ParameterConfig {
                discipline: 0,
                category: 192,
                number: 1,
                name: "Test".to_string(),
                abbreviation: "TEST".to_string(),
                unit: "K".to_string(),
                description: None,
            }],
            templates: Default::default(),
            transformations: vec![],
            backend: Default::default(),
        };

        init_registry(&config);

        let found = lookup_parameter(0, 192, 1);
        assert!(found.is_some());
        assert_eq!(found.unwrap().abbreviation, "TEST");

        clear_registry();
    }
}
