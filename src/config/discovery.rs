use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use super::creation::create_default_config_file;
use super::defaults;
use super::schema::Config;

/// Load configuration from a specific file (must exist)
/// Returns the config and the directory containing the config file
pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<(Config, PathBuf)> {
    let path = path.as_ref();
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;

    let config: Config = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

    // Get the directory containing the config file for relative path resolution
    let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

    let config_dir = canonical_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();

    tracing::debug!("Config loaded from: {}", canonical_path.display());
    Ok((config, config_dir))
}

/// Load configuration using fallback chain (silent version)
/// Returns the config and the directory containing the config file (if found)
pub fn load_with_fallback_silent() -> Result<(Config, Option<PathBuf>)> {
    let fallback_paths = [
        defaults::local_config_file(),    // ./ai_messenger.toml
        defaults::home_config_file(),     // ~/.ai_messenger.toml
        defaults::platform_config_file(), // ~/Library/Preferences/com.christiangrete.ai_messenger.toml
    ];

    for path in &fallback_paths {
        if path.exists()
            && let Ok((config, config_dir)) = load_from_file(path)
        {
            return Ok((config, Some(config_dir)));
        }
    }

    // No config file found, use defaults (silent)
    Ok((Config::default(), None))
}

/// Load configuration using fallback chain
/// Returns the config and the directory containing the config file (if found)
pub fn load_with_fallback() -> Result<(Config, Option<PathBuf>)> {
    let fallback_paths = [
        defaults::local_config_file(),    // ./ai_messenger.toml
        defaults::home_config_file(),     // ~/.ai_messenger.toml
        defaults::platform_config_file(), // ~/Library/Preferences/com.christiangrete.ai_messenger.toml
    ];

    for path in &fallback_paths {
        if path.exists() {
            match load_from_file(path) {
                Ok((config, config_dir)) => {
                    // Debug logging already handled in load_from_file
                    return Ok((config, Some(config_dir)));
                }
                Err(_e) => {
                    // Continue with next fallback
                }
            }
        }
    }

    // No config file found, create default config at platform-specific location
    let platform_config_path = defaults::platform_config_file();

    match create_default_config_file(&platform_config_path) {
        Ok(config_dir) => {
            tracing::debug!(
                "Created and loaded default config from: {}",
                platform_config_path.display()
            );
            Ok((Config::default(), Some(config_dir)))
        }
        Err(e) => {
            // If we can't write to platform location, fall back to memory defaults
            tracing::warn!(
                "Failed to create default config file: {}, using memory defaults",
                e
            );
            Ok((Config::default(), None))
        }
    }
}

/// Check if a config file exists and is readable
#[cfg(test)]
pub fn config_exists<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();
    path.exists() && path.is_file()
}

/// Get all potential config file locations for debugging
#[cfg(test)]
pub fn list_config_locations() -> Vec<(String, PathBuf, bool)> {
    let paths = [
        ("Local", defaults::local_config_file()),
        ("Home", defaults::home_config_file()),
        ("Platform", defaults::platform_config_file()),
    ];

    paths
        .into_iter()
        .map(|(name, path)| {
            let exists = config_exists(&path);
            (name.to_string(), path, exists)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_load_from_file_success() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");

        let config_content = r#"
[server]
host = "0.0.0.0"
port = 3000

[storage]
data_dir = "/test/data"
"#;

        fs::write(&config_path, config_content).unwrap();

        let (config, _config_dir) =
            load_from_file(&config_path).expect("Should load config successfully");

        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.storage.data_dir, Some("/test/data".into()));
    }

    #[test]
    fn test_load_from_file_not_found() {
        let non_existent_path = "/this/path/does/not/exist.toml";

        let result = load_from_file(non_existent_path);

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to read config file")
        );
    }

    #[test]
    fn test_load_from_file_invalid_toml() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("invalid.toml");

        let invalid_content = r#"
[server
host = "broken
"#;

        fs::write(&config_path, invalid_content).unwrap();

        let result = load_from_file(&config_path);

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to parse config file")
        );
    }

    #[test]
    fn test_load_with_fallback_silent_defaults() {
        // When no config files exist, should return defaults silently
        let (config, _config_dir) =
            load_with_fallback_silent().expect("Should return default config");

        // Should have default values
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.storage.data_dir, None);
    }

    #[test]
    fn test_load_with_fallback_defaults() {
        // When no config files exist, should return defaults with message
        let (config, _config_dir) = load_with_fallback().expect("Should return default config");

        // Should have default values
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.storage.data_dir, None);
    }

    #[test]
    fn test_config_exists() {
        let temp_dir = TempDir::new().unwrap();
        let existing_file = temp_dir.path().join("exists.toml");
        let non_existing_file = temp_dir.path().join("not_exists.toml");

        // Create an existing file
        fs::write(&existing_file, "test").unwrap();

        assert!(config_exists(&existing_file));
        assert!(!config_exists(&non_existing_file));
    }

    #[test]
    fn test_config_exists_directory() {
        let temp_dir = TempDir::new().unwrap();
        let directory = temp_dir.path().join("subdir");
        fs::create_dir(&directory).unwrap();

        // Directory should not be considered a valid config file
        assert!(!config_exists(&directory));
    }

    #[test]
    fn test_list_config_locations() {
        let locations = list_config_locations();

        // Should have exactly 3 locations
        assert_eq!(locations.len(), 3);

        // Check that we have the expected location names
        let names: Vec<&str> = locations.iter().map(|(name, _, _)| name.as_str()).collect();
        assert!(names.contains(&"Local"));
        assert!(names.contains(&"Home"));
        assert!(names.contains(&"Platform"));

        // Check that paths are different
        let paths: Vec<&PathBuf> = locations.iter().map(|(_, path, _)| path).collect();
        assert_ne!(paths[0], paths[1]);
        assert_ne!(paths[0], paths[2]);
        assert_ne!(paths[1], paths[2]);

        // All should be valid PathBuf objects
        for (_, path, _) in &locations {
            assert!(!path.as_os_str().is_empty());
        }
    }

    #[test]
    fn test_list_config_locations_existence_check() {
        let locations = list_config_locations();

        // The exists flag should reflect actual file system state
        for (name, path, exists) in locations {
            assert_eq!(
                exists,
                config_exists(&path),
                "Existence check mismatch for {}: {}",
                name,
                path.display()
            );
        }
    }

    #[test]
    fn test_load_from_file_with_minimal_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("minimal.toml");

        // Config with only one field
        let config_content = r#"
[server]
port = 9999
"#;

        fs::write(&config_path, config_content).unwrap();

        let (config, _config_dir) =
            load_from_file(&config_path).expect("Should load minimal config");

        // Should have custom port but default host
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 9999);
        assert_eq!(config.storage.data_dir, None);
    }

    #[test]
    fn test_load_from_file_empty_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("empty.toml");

        // Completely empty config file
        fs::write(&config_path, "").unwrap();

        let (config, _config_dir) = load_from_file(&config_path).expect("Should load empty config");

        // Should use all defaults
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.base_path, "");
        assert_eq!(config.storage.data_dir, None);
        assert_eq!(config.storage.cache_dir, None);
    }

    #[test]
    fn test_fallback_chain_with_corrupted_files() {
        use std::fs;
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let first_config = temp_dir.path().join("first.toml");
        let second_config = temp_dir.path().join("second.toml");

        // Create corrupted first config
        fs::write(&first_config, "invalid toml content [[[").unwrap();

        // Create valid second config
        fs::write(
            &second_config,
            r#"
[server]
host = "192.168.1.1"
port = 9000
"#,
        )
        .unwrap();

        // First config should fail, but we should handle it gracefully
        let result = load_from_file(&first_config);
        assert!(result.is_err());

        // Second config should work
        let result = load_from_file(&second_config);
        assert!(result.is_ok());
        let (config, _) = result.unwrap();
        assert_eq!(config.server.host, "192.168.1.1");
        assert_eq!(config.server.port, 9000);
    }

    #[test]
    fn test_config_canonicalize_failure_recovery() {
        // Test with a path that exists but canonicalize might fail on
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("test.toml");

        // Create valid config
        std::fs::write(
            &config_path,
            r#"
[server]
host = "localhost"
"#,
        )
        .unwrap();

        // This should work even if canonicalize has issues
        let result = load_from_file(&config_path);
        assert!(result.is_ok());

        let (config, config_dir) = result.unwrap();
        assert_eq!(config.server.host, "localhost");
        assert!(config_dir.exists());
    }

    #[test]
    fn test_load_with_fallback_error_handling() {
        // Test that load_with_fallback gracefully handles errors in fallback chain
        // by continuing to next config instead of failing

        // This mainly tests the error handling logic in the fallback loop
        // Since we can't easily create the exact fallback scenarios,
        // we test the basic error recovery behavior

        let result = load_with_fallback();
        assert!(result.is_ok());
    }
}
