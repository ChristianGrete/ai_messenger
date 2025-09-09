use anyhow::Result;
use std::path::PathBuf;

use super::{discovery, schema::Config};

/// Load configuration from file or defaults
/// Returns the config and the directory containing the config file (if found)
pub fn load_config(config_file_override: Option<String>) -> Result<(Config, Option<PathBuf>)> {
    if let Some(config_path) = config_file_override {
        // --config flag was provided - file MUST exist
        let (config, config_dir) = discovery::load_from_file(&config_path)?;
        Ok((config, Some(config_dir)))
    } else {
        // Try fallback chain
        discovery::load_with_fallback()
    }
}

/// Load configuration from file or defaults (silent version for path commands)
/// Returns the config and the directory containing the config file (if found)
pub fn load_config_silent(
    config_file_override: Option<String>,
) -> Result<(Config, Option<PathBuf>)> {
    if let Some(config_path) = config_file_override {
        // --config flag was provided - file MUST exist
        let (config, config_dir) = discovery::load_from_file(&config_path)?;
        Ok((config, Some(config_dir)))
    } else {
        // Try fallback chain (silent)
        discovery::load_with_fallback_silent()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_load_config_with_override() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.toml");

        let config_content = r#"
[server]
host = "0.0.0.0"
port = 4000

[storage]
data_dir = "/override/data"
"#;

        fs::write(&config_path, config_content).unwrap();

        let (config, config_dir) = load_config(Some(config_path.to_string_lossy().to_string()))
            .expect("Should load config from override path");

        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 4000);
        assert_eq!(config.storage.data_dir, Some("/override/data".into()));
        assert!(config_dir.is_some());
    }

    #[test]
    fn test_load_config_override_not_found() {
        let non_existent = "/this/does/not/exist.toml";

        let result = load_config(Some(non_existent.to_string()));

        assert!(result.is_err());
    }

    #[test]
    fn test_load_config_fallback() {
        // No override, should use fallback chain (which returns defaults when no files exist)
        let (config, _config_dir) = load_config(None).expect("Should return default config");

        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
    }

    #[test]
    fn test_load_config_silent_with_override() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("silent_test.toml");

        let config_content = r#"
[server]
port = 5000
"#;

        fs::write(&config_path, config_content).unwrap();

        let (config, config_dir) =
            load_config_silent(Some(config_path.to_string_lossy().to_string()))
                .expect("Should load config silently");

        assert_eq!(config.server.port, 5000);
        assert!(config_dir.is_some());
    }

    #[test]
    fn test_load_config_silent_fallback() {
        // No override, should use silent fallback (defaults)
        let (config, _config_dir) =
            load_config_silent(None).expect("Should return default config silently");

        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
    }
}
