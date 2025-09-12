use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use toml::Table;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub adapters: AdapterConfig,
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "crate::config::defaults::default_base_path")]
    pub base_path: String,
    #[serde(default = "crate::config::defaults::default_host")]
    pub host: String,
    #[serde(default = "crate::config::defaults::default_port")]
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StorageConfig {
    /// Optional override for data directory
    pub data_dir: Option<PathBuf>,
    /// Optional override for cache directory
    pub cache_dir: Option<PathBuf>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            base_path: crate::config::defaults::default_base_path(),
            host: crate::config::defaults::default_host(),
            port: crate::config::defaults::default_port(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterConfig {
    #[serde(flatten, default = "crate::config::defaults::default_adapter_services")]
    pub services: HashMap<String, ServiceAdapterConfig>,
}

impl Default for AdapterConfig {
    fn default() -> Self {
        AdapterConfig {
            services: crate::config::defaults::default_adapter_services(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAdapterConfig {
    #[serde(default = "crate::config::defaults::default_llm_provider")]
    pub provider: String,
    #[serde(default = "crate::config::defaults::default_adapter_version")]
    pub version: String,
    #[serde(default = "default_toml_value")]
    pub config: toml::Value,
}

/// Default TOML value for serde
fn default_toml_value() -> toml::Value {
    toml::Value::Table(Table::new())
}

impl ServiceAdapterConfig {
    /// Generate the default module path for this adapter
    #[allow(dead_code)]
    pub fn module_path(&self, data_dir: &Path, service: &str) -> PathBuf {
        data_dir
            .join("adapters")
            .join(service)
            .join(&self.provider)
            .join(&self.version)
            .join("adapter.wasm")
    }

    /// Generate the manifest path for this adapter
    #[allow(dead_code)]
    pub fn adapter_manifest_path(&self, data_dir: &Path, service: &str) -> PathBuf {
        data_dir
            .join("adapters")
            .join(service)
            .join(&self.provider)
            .join(&self.version)
            .join("manifest.json")
    }

    /// Get the provider config as JSON string for WASM
    #[allow(dead_code)]
    pub fn config_as_json(&self) -> Result<String, toml::ser::Error> {
        // Convert TOML value to JSON string for WASM interface
        let json_value = toml_to_json_value(&self.config);
        Ok(serde_json::to_string(&json_value).unwrap_or_else(|_| "{}".to_string()))
    }
}

impl AdapterConfig {
    /// Get adapter configuration for a specific service
    #[allow(dead_code)]
    pub fn get_service(&self, service: &str) -> Option<&ServiceAdapterConfig> {
        self.services.get(service)
    }

    /// Validate all configured adapters
    #[allow(dead_code)]
    pub fn validate(&self, data_dir: &Path) -> Result<(), AdapterValidationError> {
        for (service, config) in &self.services {
            let module_path = config.module_path(data_dir, service);

            if !module_path.exists() {
                return Err(AdapterValidationError::ModuleNotFound {
                    service: service.clone(),
                    provider: config.provider.clone(),
                    version: config.version.clone(),
                    path: module_path,
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum AdapterValidationError {
    #[error("Adapter module not found for {service} ({provider}@{version}): {path:?}")]
    ModuleNotFound {
        service: String,
        provider: String,
        version: String,
        path: PathBuf,
    },
}

/// Convert TOML value to JSON-compatible value
#[allow(dead_code)]
fn toml_to_json_value(toml_val: &toml::Value) -> serde_json::Value {
    match toml_val {
        toml::Value::String(s) => serde_json::Value::String(s.clone()),
        toml::Value::Integer(i) => serde_json::Value::Number(serde_json::Number::from(*i)),
        toml::Value::Float(f) => serde_json::Value::Number(
            serde_json::Number::from_f64(*f).unwrap_or_else(|| serde_json::Number::from(0)),
        ),
        toml::Value::Boolean(b) => serde_json::Value::Bool(*b),
        toml::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(toml_to_json_value).collect())
        }
        toml::Value::Table(table) => serde_json::Value::Object(
            table
                .iter()
                .map(|(k, v)| (k.clone(), toml_to_json_value(v)))
                .collect(),
        ),
        toml::Value::Datetime(dt) => serde_json::Value::String(dt.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();

        assert_eq!(config.server.base_path, "");
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.storage.data_dir, None);
        assert_eq!(config.storage.cache_dir, None);

        // Test adapter defaults
        assert_eq!(config.adapters.services.len(), 1);
        let llm_adapter = config
            .adapters
            .get_service("llm")
            .expect("LLM adapter should exist");
        assert_eq!(llm_adapter.provider, "ollama");
        assert_eq!(llm_adapter.version, "latest");
    }

    #[test]
    fn test_config_default_serialization() {
        let config = Config::default();
        let toml_output = toml::to_string_pretty(&config).expect("Failed to serialize");

        println!("Generated default config:");
        println!("{}", toml_output);

        // Should contain our Ollama adapter
        assert!(toml_output.contains("ollama"));
        assert!(toml_output.contains("latest"));
    }

    #[test]
    fn test_config_serde_full() {
        let toml_content = r#"
[server]
base_path = "api"
host = "0.0.0.0"
port = 3000

[storage]
data_dir = "/custom/data"
cache_dir = "/custom/cache"
"#;

        let config: Config = toml::from_str(toml_content).expect("Failed to parse TOML");

        assert_eq!(config.server.base_path, "api");
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.storage.data_dir, Some("/custom/data".into()));
        assert_eq!(config.storage.cache_dir, Some("/custom/cache".into()));
    }

    #[test]
    fn test_config_partial() {
        let toml_content = r#"
[server]
port = 9000
"#;

        let config: Config = toml::from_str(toml_content).expect("Failed to parse partial TOML");

        // Should use defaults for missing fields
        assert_eq!(config.server.base_path, "");
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 9000);
        assert_eq!(config.storage.data_dir, None);
        assert_eq!(config.storage.cache_dir, None);
    }

    #[test]
    fn test_config_empty() {
        let toml_content = "";

        let config: Config = toml::from_str(toml_content).expect("Failed to parse empty TOML");

        // Should use all defaults
        assert_eq!(config.server.base_path, "");
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.storage.data_dir, None);
        assert_eq!(config.storage.cache_dir, None);
    }

    #[test]
    fn test_config_storage_only() {
        let toml_content = r#"
[storage]
data_dir = "/my/data"
"#;

        let config: Config =
            toml::from_str(toml_content).expect("Failed to parse storage-only TOML");

        // Server should use defaults
        assert_eq!(config.server.base_path, "");
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        // Storage should have custom data_dir
        assert_eq!(config.storage.data_dir, Some("/my/data".into()));
        assert_eq!(config.storage.cache_dir, None);
    }

    #[test]
    fn test_config_invalid_toml() {
        let invalid_toml = r#"
[server
host = "invalid
"#;

        let result = toml::from_str::<Config>(invalid_toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_invalid_port() {
        let invalid_toml = r#"
[server]
port = "not_a_number"
"#;

        let result = toml::from_str::<Config>(invalid_toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let original = Config {
            server: ServerConfig {
                base_path: "api".to_string(),
                host: "0.0.0.0".to_string(),
                port: 3000,
            },
            storage: StorageConfig {
                data_dir: Some("/test/data".into()),
                cache_dir: Some("/test/cache".into()),
            },
            ..Config::default()
        };

        // Serialize to TOML
        let toml_string = toml::to_string(&original).expect("Failed to serialize to TOML");

        // Deserialize back
        let deserialized: Config =
            toml::from_str(&toml_string).expect("Failed to deserialize from TOML");

        // Should be identical
        assert_eq!(original.server.base_path, deserialized.server.base_path);
        assert_eq!(original.server.host, deserialized.server.host);
        assert_eq!(original.server.port, deserialized.server.port);
        assert_eq!(original.storage.data_dir, deserialized.storage.data_dir);
        assert_eq!(original.storage.cache_dir, deserialized.storage.cache_dir);
    }

    #[test]
    fn test_adapter_config_parsing() {
        let toml_content = r#"
[adapters.llm]
provider = "openai"
version = "v1"

[adapters.llm.config]
api_key = "sk-123456"
model = "gpt-4"

[adapters.storage]
provider = "s3"
version = "2.0"

[adapters.storage.config]
bucket = "my-bucket"
region = "us-east-1"
"#;

        let config: Config = toml::from_str(toml_content).expect("Failed to parse adapter TOML");

        // Test LLM adapter
        let llm_adapter = config.adapters.get_service("llm").unwrap();
        assert_eq!(llm_adapter.provider, "openai");
        assert_eq!(llm_adapter.version, "v1");

        // Test storage adapter
        let storage_adapter = config.adapters.get_service("storage").unwrap();
        assert_eq!(storage_adapter.provider, "s3");
        assert_eq!(storage_adapter.version, "2.0");

        // Should have 2 adapters total
        assert_eq!(config.adapters.services.len(), 2);
    }

    #[test]
    fn test_adapter_config_partial_defaults() {
        let toml_content = r#"
[adapters.llm]
# Only provider specified, version and config should use defaults
provider = "custom-llm"
"#;

        let config: Config =
            toml::from_str(toml_content).expect("Failed to parse partial adapter TOML");

        let llm_adapter = config.adapters.get_service("llm").unwrap();
        assert_eq!(llm_adapter.provider, "custom-llm");
        assert_eq!(llm_adapter.version, "latest"); // Should use default

        // Config should be empty table (default)
        match &llm_adapter.config {
            toml::Value::Table(table) => assert!(table.is_empty()),
            _ => panic!("Expected empty TOML table"),
        }
    }

    #[test]
    fn test_adapter_config_empty_uses_defaults() {
        let toml_content = r#"
[server]
port = 3000
"#;

        let config: Config =
            toml::from_str(toml_content).expect("Failed to parse config without adapters");

        // Should still have default Ollama adapter
        assert_eq!(config.adapters.services.len(), 1);
        let llm_adapter = config.adapters.get_service("llm").unwrap();
        assert_eq!(llm_adapter.provider, "ollama");
        assert_eq!(llm_adapter.version, "latest");
    }

    #[test]
    fn test_service_adapter_config_defaults() {
        let toml_content = r#"
[adapters.test]
# All fields should use defaults
"#;

        let config: Config =
            toml::from_str(toml_content).expect("Failed to parse empty adapter config");

        let test_adapter = config.adapters.get_service("test").unwrap();
        assert_eq!(test_adapter.provider, "ollama"); // Default LLM provider
        assert_eq!(test_adapter.version, "latest"); // Default version

        match &test_adapter.config {
            toml::Value::Table(table) => assert!(table.is_empty()),
            _ => panic!("Expected empty TOML table"),
        }
    }

    #[test]
    fn test_adapter_module_path_generation() {
        let adapter = ServiceAdapterConfig {
            provider: "ollama".to_string(),
            version: "1.0.0".to_string(),
            config: toml::Value::Table(Table::new()),
        };

        let data_dir = std::path::Path::new("/data");
        let module_path = adapter.module_path(data_dir, "llm");

        let expected = std::path::PathBuf::from("/data/adapters/llm/ollama/1.0.0/adapter.wasm");
        assert_eq!(module_path, expected);
    }

    #[test]
    fn test_adapter_manifest_path_generation() {
        let adapter = ServiceAdapterConfig {
            provider: "ollama".to_string(),
            version: "1.0.0".to_string(),
            config: toml::Value::Table(Table::new()),
        };

        let data_dir = std::path::Path::new("/data");
        let manifest_path = adapter.adapter_manifest_path(data_dir, "llm");

        let expected = std::path::PathBuf::from("/data/adapters/llm/ollama/1.0.0/manifest.json");
        assert_eq!(manifest_path, expected);
    }

    #[test]
    fn test_adapter_config_as_json() {
        let mut config_table = Table::new();
        config_table.insert(
            "api_key".to_string(),
            toml::Value::String("test-key".to_string()),
        );
        config_table.insert("timeout".to_string(), toml::Value::Integer(30));
        config_table.insert("enabled".to_string(), toml::Value::Boolean(true));

        let adapter = ServiceAdapterConfig {
            provider: "test".to_string(),
            version: "1.0".to_string(),
            config: toml::Value::Table(config_table),
        };

        let json_result = adapter.config_as_json().expect("Failed to convert to JSON");
        let parsed_json: serde_json::Value =
            serde_json::from_str(&json_result).expect("Generated JSON should be valid");

        assert_eq!(parsed_json["api_key"], "test-key");
        assert_eq!(parsed_json["timeout"], 30);
        assert_eq!(parsed_json["enabled"], true);
    }

    #[test]
    fn test_toml_to_json_conversion() {
        // Test different TOML value types
        let test_cases = vec![
            (
                toml::Value::String("test".to_string()),
                serde_json::Value::String("test".to_string()),
            ),
            (
                toml::Value::Integer(42),
                serde_json::Value::Number(serde_json::Number::from(42)),
            ),
            (toml::Value::Boolean(true), serde_json::Value::Bool(true)),
        ];

        for (toml_val, expected_json) in test_cases {
            let result = toml_to_json_value(&toml_val);
            assert_eq!(result, expected_json);
        }
    }
}
