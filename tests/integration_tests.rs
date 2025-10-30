//! Integration tests for gribberish refactoring
//!
//! Tests that verify all components work together correctly

use gribberish::api::{build_message_index, filter_messages_by_variable, read_all_messages};
use gribberish::backends::{get_backend, BackendType};
use gribberish::config::{clear_registry, init_registry, lookup_parameter, ConfigLoader};

#[test]
fn test_end_to_end_with_config() {
    // Clear any previous state
    clear_registry();

    // Create a test configuration
    let yaml = r#"
version: "1.0"
parameters:
  - discipline: 0
    category: 192
    number: 1
    name: "Test Parameter"
    abbreviation: "TESTP"
    unit: "K"
backend:
  preferred: "native"
  fallback: true
"#;

    // Load configuration
    let config = ConfigLoader::from_yaml_str(yaml).expect("Failed to parse config");

    // Initialize registry
    init_registry(&config);

    // Verify parameter is registered
    let param = lookup_parameter(0, 192, 1);
    assert!(param.is_some());
    assert_eq!(param.unwrap().abbreviation, "TESTP");

    // Test that we can use the configured backend
    let backend = get_backend(BackendType::Native);
    assert_eq!(backend.name(), "native");

    // Cleanup
    clear_registry();
}

#[test]
fn test_config_and_api_integration() {
    clear_registry();

    // Load a configuration
    let yaml = r#"
version: "1.0"
parameters:
  - discipline: 0
    category: 0
    number: 0
    name: "Temperature"
    abbreviation: "TMP"
    unit: "K"
transformations:
  - variable: "TMP"
    operations:
      - type: "scale"
        factor: 1.8
        offset: -459.67
"#;

    let config = ConfigLoader::from_yaml_str(yaml).expect("Failed to parse");
    init_registry(&config);

    // Now use the API with empty data (just testing integration)
    let data = b"";
    let messages = read_all_messages(data, BackendType::Native);

    assert!(messages.is_ok());

    clear_registry();
}

#[test]
fn test_multiple_backends_same_data() {
    let data = b"";

    // Test that both backends handle the same data consistently
    let native_messages = read_all_messages(data, BackendType::Native);
    assert!(native_messages.is_ok());

    #[cfg(feature = "eccodes")]
    {
        let eccodes_messages = read_all_messages(data, BackendType::Eccodes);
        assert!(eccodes_messages.is_ok());

        // Both should return empty for empty data
        assert_eq!(
            native_messages.unwrap().len(),
            eccodes_messages.unwrap().len()
        );
    }
}

#[test]
fn test_config_file_formats() {
    // Test that YAML and JSON produce equivalent configs
    let yaml = r#"
version: "1.0"
parameters:
  - discipline: 0
    category: 192
    number: 1
    name: "Test"
    abbreviation: "TEST"
    unit: "K"
"#;

    let json = r#"{
  "version": "1.0",
  "parameters": [
    {
      "discipline": 0,
      "category": 192,
      "number": 1,
      "name": "Test",
      "abbreviation": "TEST",
      "unit": "K"
    }
  ]
}"#;

    let yaml_config = ConfigLoader::from_yaml_str(yaml).expect("YAML failed");
    let json_config = ConfigLoader::from_json_str(json).expect("JSON failed");

    assert_eq!(yaml_config.version, json_config.version);
    assert_eq!(yaml_config.parameters.len(), json_config.parameters.len());
    assert_eq!(
        yaml_config.parameters[0].abbreviation,
        json_config.parameters[0].abbreviation
    );
}

#[test]
fn test_config_merge_and_registry() {
    clear_registry();

    let config1_yaml = r#"
parameters:
  - discipline: 0
    category: 192
    number: 1
    name: "Param1"
    abbreviation: "P1"
    unit: "K"
"#;

    let config2_yaml = r#"
parameters:
  - discipline: 0
    category: 192
    number: 2
    name: "Param2"
    abbreviation: "P2"
    unit: "m/s"
"#;

    let config1 = ConfigLoader::from_yaml_str(config1_yaml).expect("Failed");
    let config2 = ConfigLoader::from_yaml_str(config2_yaml).expect("Failed");

    // Merge configs
    let merged = ConfigLoader::merge(vec![config1, config2]);

    // Initialize registry with merged config
    init_registry(&merged);

    // Both parameters should be available
    assert!(lookup_parameter(0, 192, 1).is_some());
    assert!(lookup_parameter(0, 192, 2).is_some());

    clear_registry();
}

#[test]
fn test_api_functions_consistency() {
    let data = b"Not a GRIB file";

    // All API functions should handle invalid data gracefully
    let read_result = read_all_messages(data, BackendType::Native);
    let index_result = build_message_index(data, BackendType::Native);
    let filter_result = filter_messages_by_variable(data, "TMP", BackendType::Native);

    // Should all succeed (returning empty results)
    assert!(read_result.is_ok());
    assert!(index_result.is_ok());
    assert!(filter_result.is_ok());

    // All should be empty
    assert!(read_result.unwrap().is_empty());
    assert!(index_result.unwrap().is_empty());
    assert!(filter_result.unwrap().is_empty());
}

#[test]
fn test_backend_switching() {
    let data = b"";

    // Should be able to switch backends multiple times
    for _ in 0..3 {
        let native_result = read_all_messages(data, BackendType::Native);
        assert!(native_result.is_ok());

        #[cfg(feature = "eccodes")]
        {
            let eccodes_result = read_all_messages(data, BackendType::Eccodes);
            assert!(eccodes_result.is_ok());
        }
    }
}

#[test]
fn test_config_with_all_features() {
    clear_registry();

    let yaml = r#"
version: "1.0"
parameters:
  - discipline: 0
    category: 192
    number: 1
    name: "Custom Temperature"
    abbreviation: "CTEMP"
    unit: "K"
    description: "A custom temperature parameter"
templates:
  product:
    - number: 1000
      name: "CustomTemplate"
      description: "Custom product template"
      fields:
        - name: "category"
          offset: 9
          length: 1
          fieldType: "u8"
transformations:
  - variable: "CTEMP"
    operations:
      - type: "scale"
        factor: 1.8
        offset: -459.67
backend:
  preferred: "native"
  fallback: true
"#;

    let config = ConfigLoader::from_yaml_str(yaml).expect("Failed to parse");

    // Verify all parts of the config
    assert_eq!(config.parameters.len(), 1);
    assert_eq!(config.templates.product.len(), 1);
    assert_eq!(config.transformations.len(), 1);
    assert_eq!(config.backend.preferred, "native");

    // Initialize and test registry
    init_registry(&config);

    let param = lookup_parameter(0, 192, 1);
    assert!(param.is_some());
    assert_eq!(param.unwrap().abbreviation, "CTEMP");

    clear_registry();
}

#[test]
fn test_error_handling_chain() {
    // Test that errors propagate correctly through the stack

    // Invalid GRIB data
    let data = b"INVALID";

    // Backend should handle it
    let backend = get_backend(BackendType::Native);
    let parse_result = backend.parse_message(data, 0);
    assert!(parse_result.is_err());

    // API should handle it
    let api_result = read_all_messages(data, BackendType::Native);
    // API returns Ok with empty vector for unparseable data
    assert!(api_result.is_ok());
}

#[test]
fn test_thread_safety_of_registry() {
    use std::sync::Arc;
    use std::thread;
    use std::sync::Barrier;

    clear_registry();

    // Initialize registry
    let yaml = r#"
parameters:
  - discipline: 0
    category: 192
    number: 1
    name: "ThreadTest"
    abbreviation: "TTEST"
    unit: "K"
"#;

    let config = ConfigLoader::from_yaml_str(yaml).expect("Failed");
    init_registry(&config);

    // Use a barrier to ensure all threads start at the same time
    let barrier = Arc::new(Barrier::new(5));

    // Spawn multiple threads that read from the registry
    let handles: Vec<_> = (0..5)
        .map(|_| {
            let barrier_clone = Arc::clone(&barrier);
            thread::spawn(move || {
                // Wait for all threads to be ready
                barrier_clone.wait();

                // Each thread should be able to lookup parameters
                for _ in 0..100 {
                    let param = lookup_parameter(0, 192, 1);
                    if param.is_none() {
                        // Registry might not be initialized yet, but that's ok
                        // Just don't panic - this tests thread safety, not timing
                        continue;
                    }
                    assert_eq!(param.unwrap().abbreviation, "TTEST");
                }
            })
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    clear_registry();
}

#[test]
fn test_backward_compatibility() {
    // Test that the old API still works
    use gribberish::message::{read_message, read_messages, scan_messages};

    let data = b"";

    // Old functions should still work
    let messages = read_messages(data);
    let count = messages.count();
    assert_eq!(count, 0);

    let message = read_message(data, 0);
    assert!(message.is_none());

    let scanned = scan_messages(data);
    assert_eq!(scanned.len(), 0);
}

#[test]
fn test_config_validation() {
    // Test that invalid configs are rejected appropriately

    // Missing required fields
    let invalid_yaml = r#"
parameters:
  - discipline: 0
    # Missing category, number, name, abbreviation, unit
"#;

    let result = ConfigLoader::from_yaml_str(invalid_yaml);
    assert!(result.is_err(), "Should reject config with missing fields");

    // Valid minimal config
    let valid_yaml = r#"
parameters:
  - discipline: 0
    category: 0
    number: 0
    name: "Test"
    abbreviation: "T"
    unit: "K"
"#;

    let result = ConfigLoader::from_yaml_str(valid_yaml);
    assert!(result.is_ok(), "Should accept valid minimal config");
}

#[cfg(feature = "eccodes")]
mod eccodes_integration {
    use super::*;

    #[test]
    fn test_eccodes_backend_integration() {
        let data = b"";

        // Test that eccodes backend integrates with API
        let result = read_all_messages(data, BackendType::Eccodes);
        assert!(result.is_ok());

        let backend = get_backend(BackendType::Eccodes);
        assert_eq!(backend.name(), "eccodes");
    }

    #[test]
    fn test_eccodes_with_config() {
        clear_registry();

        let yaml = r#"
version: "1.0"
backend:
  preferred: "eccodes"
  fallback: true
"#;

        let config = ConfigLoader::from_yaml_str(yaml).expect("Failed");
        assert_eq!(config.backend.preferred, "eccodes");

        clear_registry();
    }
}
