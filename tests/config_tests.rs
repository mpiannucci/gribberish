//! Tests for the configuration system
//!
//! Tests schema, loader, and registry functionality

use gribberish::config::{
    clear_registry, init_registry, lookup_parameter, ConfigLoader, GribConfig, OperationConfig,
    ParameterConfig, ParameterRegistry,
};

#[test]
fn test_parameter_config_creation() {
    let param = ParameterConfig {
        discipline: 0,
        category: 192,
        number: 1,
        name: "Test Parameter".to_string(),
        abbreviation: "TEST".to_string(),
        unit: "K".to_string(),
        description: Some("A test parameter".to_string()),
    };

    assert_eq!(param.discipline, 0);
    assert_eq!(param.category, 192);
    assert_eq!(param.abbreviation, "TEST");
}

#[test]
fn test_yaml_deserialization() {
    let yaml = r#"
version: "1.0"
parameters:
  - discipline: 0
    category: 192
    number: 1
    name: "Custom Temperature"
    abbreviation: "CTEMP"
    unit: "K"
    description: "Test parameter"
backend:
  preferred: "native"
  fallback: true
"#;

    let config = ConfigLoader::from_yaml_str(yaml).expect("Failed to parse YAML");

    assert_eq!(config.version, "1.0");
    assert_eq!(config.parameters.len(), 1);
    assert_eq!(config.parameters[0].abbreviation, "CTEMP");
    assert_eq!(config.backend.preferred, "native");
    assert!(config.backend.fallback);
}

#[test]
fn test_json_deserialization() {
    let json = r#"{
  "version": "1.0",
  "parameters": [
    {
      "discipline": 0,
      "category": 192,
      "number": 1,
      "name": "Custom Temperature",
      "abbreviation": "CTEMP",
      "unit": "K"
    }
  ],
  "backend": {
    "preferred": "native",
    "fallback": true
  }
}"#;

    let config = ConfigLoader::from_json_str(json).expect("Failed to parse JSON");

    assert_eq!(config.version, "1.0");
    assert_eq!(config.parameters.len(), 1);
    assert_eq!(config.parameters[0].abbreviation, "CTEMP");
}

#[test]
fn test_yaml_with_defaults() {
    let yaml = r#"
parameters:
  - discipline: 0
    category: 192
    number: 1
    name: "Test"
    abbreviation: "TEST"
    unit: "K"
"#;

    let config = ConfigLoader::from_yaml_str(yaml).expect("Failed to parse YAML");

    // Should use default version
    assert_eq!(config.version, "1.0");

    // Should use default backend
    assert_eq!(config.backend.preferred, "auto");
    assert!(config.backend.fallback);
}

#[test]
fn test_invalid_yaml() {
    let yaml = r#"
invalid: yaml: structure:
  - this is not valid
"#;

    let result = ConfigLoader::from_yaml_str(yaml);
    assert!(result.is_err());
}

#[test]
fn test_config_with_transformations() {
    let yaml = r#"
version: "1.0"
transformations:
  - variable: "TMP"
    operations:
      - type: "scale"
        factor: 1.8
        offset: -459.67
      - type: "clip"
        min: 0.0
        max: 500.0
"#;

    let config = ConfigLoader::from_yaml_str(yaml).expect("Failed to parse YAML");

    assert_eq!(config.transformations.len(), 1);
    assert_eq!(config.transformations[0].variable, "TMP");
    assert_eq!(config.transformations[0].operations.len(), 2);

    match &config.transformations[0].operations[0] {
        OperationConfig::Scale { factor, offset } => {
            assert_eq!(*factor, 1.8);
            assert_eq!(*offset, -459.67);
        }
        _ => panic!("Expected Scale operation"),
    }

    match &config.transformations[0].operations[1] {
        OperationConfig::Clip { min, max } => {
            assert_eq!(*min, Some(0.0));
            assert_eq!(*max, Some(500.0));
        }
        _ => panic!("Expected Clip operation"),
    }
}

#[test]
fn test_parameter_registry() {
    let mut registry = ParameterRegistry::new();

    let param = ParameterConfig {
        discipline: 0,
        category: 192,
        number: 1,
        name: "Test".to_string(),
        abbreviation: "TEST".to_string(),
        unit: "K".to_string(),
        description: None,
    };

    registry.register(param);

    // Should be able to look up the parameter
    let found = registry.lookup(0, 192, 1);
    assert!(found.is_some());
    assert_eq!(found.unwrap().abbreviation, "TEST");

    // Should not find non-existent parameter
    let not_found = registry.lookup(0, 192, 2);
    assert!(not_found.is_none());

    // Check count
    assert_eq!(registry.count(), 1);
}

#[test]
fn test_parameter_registry_multiple() {
    let mut registry = ParameterRegistry::new();

    let params = vec![
        ParameterConfig {
            discipline: 0,
            category: 192,
            number: 1,
            name: "Test1".to_string(),
            abbreviation: "TEST1".to_string(),
            unit: "K".to_string(),
            description: None,
        },
        ParameterConfig {
            discipline: 0,
            category: 192,
            number: 2,
            name: "Test2".to_string(),
            abbreviation: "TEST2".to_string(),
            unit: "m/s".to_string(),
            description: None,
        },
    ];

    registry.register_many(params);

    assert_eq!(registry.count(), 2);

    let all = registry.all_parameters();
    assert_eq!(all.len(), 2);
}

#[test]
fn test_parameter_registry_clear() {
    let mut registry = ParameterRegistry::new();

    let param = ParameterConfig {
        discipline: 0,
        category: 192,
        number: 1,
        name: "Test".to_string(),
        abbreviation: "TEST".to_string(),
        unit: "K".to_string(),
        description: None,
    };

    registry.register(param);
    assert_eq!(registry.count(), 1);

    registry.clear();
    assert_eq!(registry.count(), 0);
}

#[test]
fn test_global_registry() {
    // Clear any previous state
    clear_registry();

    let config = GribConfig {
        version: "1.0".to_string(),
        parameters: vec![ParameterConfig {
            discipline: 0,
            category: 192,
            number: 1,
            name: "Global Test".to_string(),
            abbreviation: "GTEST".to_string(),
            unit: "K".to_string(),
            description: None,
        }],
        templates: Default::default(),
        transformations: vec![],
        backend: Default::default(),
    };

    init_registry(&config);

    // Should be able to look up in global registry
    let found = lookup_parameter(0, 192, 1);
    assert!(found.is_some());
    assert_eq!(found.unwrap().abbreviation, "GTEST");

    // Cleanup
    clear_registry();
}

#[test]
fn test_config_merge() {
    let config1 = GribConfig {
        version: "1.0".to_string(),
        parameters: vec![ParameterConfig {
            discipline: 0,
            category: 192,
            number: 1,
            name: "Param1".to_string(),
            abbreviation: "P1".to_string(),
            unit: "K".to_string(),
            description: None,
        }],
        templates: Default::default(),
        transformations: vec![],
        backend: Default::default(),
    };

    let mut config2 = config1.clone();
    config2.parameters = vec![ParameterConfig {
        discipline: 0,
        category: 192,
        number: 2,
        name: "Param2".to_string(),
        abbreviation: "P2".to_string(),
        unit: "m/s".to_string(),
        description: None,
    }];
    config2.backend.preferred = "eccodes".to_string();

    let merged = ConfigLoader::merge(vec![config1, config2]);

    // Should have both parameters
    assert_eq!(merged.parameters.len(), 2);

    // Should use last backend config
    assert_eq!(merged.backend.preferred, "eccodes");
}

#[test]
fn test_config_with_templates() {
    let yaml = r#"
version: "1.0"
templates:
  product:
    - number: 1000
      name: "CustomTemplate"
      description: "A custom template"
      fields:
        - name: "test_field"
          offset: 0
          length: 4
          fieldType: "u32"
  gridDefinition:
    - number: 2000
      name: "CustomGrid"
      projection: "+proj=custom"
      fields:
        - name: "nx"
          offset: 0
          length: 4
"#;

    let config = ConfigLoader::from_yaml_str(yaml).expect("Failed to parse YAML");

    assert_eq!(config.templates.product.len(), 1);
    assert_eq!(config.templates.product[0].number, 1000);
    assert_eq!(config.templates.product[0].fields.len(), 1);

    assert_eq!(config.templates.grid_definition.len(), 1);
    assert_eq!(config.templates.grid_definition[0].number, 2000);
}

#[test]
fn test_yaml_with_all_operation_types() {
    let yaml = r#"
transformations:
  - variable: "TEST"
    operations:
      - type: "scale"
        factor: 2.0
        offset: 10.0
      - type: "unitConversion"
        from: "K"
        to: "C"
      - type: "formula"
        expression: "x * 2 + 5"
      - type: "clip"
        min: 0.0
        max: 100.0
"#;

    let config = ConfigLoader::from_yaml_str(yaml).expect("Failed to parse YAML");

    assert_eq!(config.transformations.len(), 1);
    assert_eq!(config.transformations[0].operations.len(), 4);

    // Verify all operation types are parsed correctly
    match &config.transformations[0].operations[0] {
        OperationConfig::Scale { .. } => (),
        _ => panic!("Expected Scale"),
    }

    match &config.transformations[0].operations[1] {
        OperationConfig::UnitConversion { .. } => (),
        _ => panic!("Expected UnitConversion"),
    }

    match &config.transformations[0].operations[2] {
        OperationConfig::Formula { .. } => (),
        _ => panic!("Expected Formula"),
    }

    match &config.transformations[0].operations[3] {
        OperationConfig::Clip { .. } => (),
        _ => panic!("Expected Clip"),
    }
}

#[test]
fn test_empty_config() {
    let yaml = r#"
version: "1.0"
"#;

    let config = ConfigLoader::from_yaml_str(yaml).expect("Failed to parse YAML");

    assert_eq!(config.parameters.len(), 0);
    assert_eq!(config.transformations.len(), 0);
    assert_eq!(config.templates.product.len(), 0);
}

#[test]
fn test_standard_locations_no_files() {
    // This should return a default config if no files are found
    let config = ConfigLoader::from_standard_locations();

    assert_eq!(config.version, "1.0");
    assert_eq!(config.parameters.len(), 0);
}
