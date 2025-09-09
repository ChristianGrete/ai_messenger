use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
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
}
