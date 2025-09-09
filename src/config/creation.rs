use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use super::schema::Config;

/// Create a default config file at the specified path
/// Returns the directory containing the created config file
pub fn create_default_config_file<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    let path = path.as_ref();

    // Create parent directory if it doesn't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
    }

    // Generate default config
    let default_config = Config::default();
    let toml_content = toml::to_string_pretty(&default_config)
        .context("Failed to serialize default config to TOML")?;

    // Write config file
    fs::write(path, toml_content)
        .with_context(|| format!("Failed to write default config file: {}", path.display()))?;

    tracing::info!("Created default config file: {}", path.display());

    // Return the directory containing the config file
    let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    Ok(canonical_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf())
}

/// Create a config file with custom content
/// Returns the directory containing the created config file
#[allow(dead_code)]
pub fn create_config_file<P: AsRef<Path>>(path: P, config: &Config) -> Result<PathBuf> {
    let path = path.as_ref();

    // Create parent directory if it doesn't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
    }

    // Serialize config to TOML
    let toml_content =
        toml::to_string_pretty(config).context("Failed to serialize config to TOML")?;

    // Write config file
    fs::write(path, toml_content)
        .with_context(|| format!("Failed to write config file: {}", path.display()))?;

    tracing::info!("Created config file: {}", path.display());

    // Return the directory containing the config file
    let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    Ok(canonical_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_create_default_config_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_default.toml");

        // Create default config file
        let result = create_default_config_file(&config_path);
        assert!(result.is_ok());

        // Verify file was created
        assert!(config_path.exists());

        // Verify file content is valid TOML and has expected defaults
        let content = fs::read_to_string(&config_path).unwrap();
        let parsed_config: Config = toml::from_str(&content).unwrap();

        assert_eq!(parsed_config.server.host, "127.0.0.1");
        assert_eq!(parsed_config.server.port, 8080);
        assert_eq!(parsed_config.server.base_path, "");
        assert_eq!(parsed_config.storage.data_dir, None);
        assert_eq!(parsed_config.storage.cache_dir, None);
    }

    #[test]
    fn test_create_default_config_file_with_nested_directory() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir
            .path()
            .join("deeply")
            .join("nested")
            .join("config.toml");

        // Should create parent directories automatically
        let result = create_default_config_file(&nested_path);
        assert!(result.is_ok());

        // Verify file and directories were created
        assert!(nested_path.exists());
        assert!(nested_path.parent().unwrap().exists());
    }

    #[test]
    fn test_create_config_file_with_custom_content() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("custom.toml");

        // Create custom config
        let mut custom_config = Config::default();
        custom_config.server.host = "192.168.1.1".to_string();
        custom_config.server.port = 9000;
        custom_config.storage.data_dir = Some("/custom/data".into());

        let result = create_config_file(&config_path, &custom_config);
        assert!(result.is_ok());

        // Verify file was created with custom content
        assert!(config_path.exists());

        let content = fs::read_to_string(&config_path).unwrap();
        let parsed_config: Config = toml::from_str(&content).unwrap();

        assert_eq!(parsed_config.server.host, "192.168.1.1");
        assert_eq!(parsed_config.server.port, 9000);
        assert_eq!(parsed_config.storage.data_dir, Some("/custom/data".into()));
    }

    #[test]
    fn test_create_config_file_nested_directories() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir
            .path()
            .join("very")
            .join("deeply")
            .join("nested")
            .join("custom.toml");

        let custom_config = Config::default();
        let result = create_config_file(&nested_path, &custom_config);
        assert!(result.is_ok());

        // Verify file and all parent directories were created
        assert!(nested_path.exists());
        assert!(nested_path.parent().unwrap().exists());
    }
}
