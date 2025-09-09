use std::path::PathBuf;

use super::{defaults, path_expansion, schema::Config};

/// Get the effective data directory for storing persistent data
pub fn data_dir(config: &Config, config_dir: Option<&std::path::Path>) -> PathBuf {
    expand_optional_path(config.storage.data_dir.as_ref(), config_dir, || {
        defaults::default_data_dir()
    })
}

/// Get the effective cache directory for storing temporary data
pub fn cache_dir(config: &Config, config_dir: Option<&std::path::Path>) -> PathBuf {
    expand_optional_path(config.storage.cache_dir.as_ref(), config_dir, || {
        defaults::default_cache_dir()
    })
}

/// Generic helper to expand an optional path from config or use a default
///
/// This can be used for any optional path configuration that should support
/// home directory expansion and config-relative paths. Example usage:
///
/// ```rust,no_run
/// use std::path::PathBuf;
/// use ai_messenger::config::paths::expand_optional_path;
///
/// let config_dir = Some(PathBuf::from("/etc/app"));
/// let log_file_config: Option<PathBuf> = Some(PathBuf::from("~/logs/app.log"));
///
/// let log_file = expand_optional_path(
///     log_file_config.as_ref(),
///     config_dir.as_deref(),
///     || PathBuf::from("/var/log/app.log")  // default
/// );
/// ```
pub fn expand_optional_path<F>(
    config_path: Option<&PathBuf>,
    config_dir: Option<&std::path::Path>,
    default_fn: F,
) -> PathBuf
where
    F: FnOnce() -> PathBuf,
{
    config_path
        .map(|path| path_expansion::expand_path(path, config_dir))
        .unwrap_or_else(default_fn)
}

/// Generic helper to expand a path that's always present
///
/// This can be used when a path is mandatory in config and should always
/// be expanded. Example usage:
///
/// ```rust,no_run
/// use std::path::PathBuf;
/// use ai_messenger::config::paths::expand_required_path;
///
/// let config_dir = Some(PathBuf::from("/etc/app"));
/// let required_path = PathBuf::from("~/config/settings.json");
///
/// let expanded_path = expand_required_path(&required_path, config_dir.as_deref());
/// ```
#[allow(dead_code)]
pub fn expand_required_path<P: AsRef<std::path::Path>>(
    path: P,
    config_dir: Option<&std::path::Path>,
) -> PathBuf {
    path_expansion::expand_path(path, config_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::schema;
    use std::path::PathBuf;

    #[test]
    fn test_data_dir_with_config_override() {
        let config = Config {
            server: schema::ServerConfig::default(),
            storage: schema::StorageConfig {
                data_dir: Some("/custom/data".into()),
                cache_dir: None,
            },
        };

        let result_dir = data_dir(&config, None);
        assert_eq!(result_dir, PathBuf::from("/custom/data"));
    }

    #[test]
    fn test_data_dir_with_defaults() {
        let config = Config::default();

        let result_dir = data_dir(&config, None);
        let expected_dir = defaults::default_data_dir();

        assert_eq!(result_dir, expected_dir);
    }

    #[test]
    fn test_cache_dir_with_config_override() {
        let config = Config {
            server: schema::ServerConfig::default(),
            storage: schema::StorageConfig {
                data_dir: None,
                cache_dir: Some("/custom/cache".into()),
            },
        };

        let result_dir = cache_dir(&config, None);
        assert_eq!(result_dir, PathBuf::from("/custom/cache"));
    }

    #[test]
    fn test_cache_dir_with_defaults() {
        let config = Config::default();

        let result_dir = cache_dir(&config, None);
        let expected_dir = defaults::default_cache_dir();

        assert_eq!(result_dir, expected_dir);
    }

    #[test]
    fn test_data_and_cache_dir_independence() {
        let config = Config {
            server: schema::ServerConfig::default(),
            storage: schema::StorageConfig {
                data_dir: Some("/custom/data".into()),
                cache_dir: Some("/custom/cache".into()),
            },
        };

        let data = data_dir(&config, None);
        let cache = cache_dir(&config, None);

        assert_eq!(data, PathBuf::from("/custom/data"));
        assert_eq!(cache, PathBuf::from("/custom/cache"));
        assert_ne!(data, cache);
    }

    #[test]
    fn test_data_dir_with_tilde_expansion() {
        let config = Config {
            server: schema::ServerConfig::default(),
            storage: schema::StorageConfig {
                data_dir: Some("~/custom/data".into()),
                cache_dir: None,
            },
        };

        let result_dir = data_dir(&config, None);

        // Should not contain ~ anymore
        assert!(!result_dir.to_string_lossy().contains("~"));
        // Should end with the expected path
        assert!(result_dir.to_string_lossy().ends_with("/custom/data"));
    }

    #[test]
    fn test_cache_dir_with_dollar_home_expansion() {
        let config = Config {
            server: schema::ServerConfig::default(),
            storage: schema::StorageConfig {
                data_dir: None,
                cache_dir: Some("$HOME/.cache/ai_messenger".into()),
            },
        };

        let result_dir = cache_dir(&config, None);

        // Should not contain $HOME anymore
        assert!(!result_dir.to_string_lossy().contains("$HOME"));
        // Should end with the expected path
        assert!(
            result_dir
                .to_string_lossy()
                .ends_with("/.cache/ai_messenger")
        );
    }

    #[test]
    fn test_data_dir_absolute_path_unchanged() {
        let config = Config {
            server: schema::ServerConfig::default(),
            storage: schema::StorageConfig {
                data_dir: Some("/absolute/path/data".into()),
                cache_dir: None,
            },
        };

        let result_dir = data_dir(&config, None);

        // Absolute paths should remain unchanged
        assert_eq!(result_dir, PathBuf::from("/absolute/path/data"));
    }

    #[test]
    fn test_config_with_both_expansions() {
        let config = Config {
            server: schema::ServerConfig::default(),
            storage: schema::StorageConfig {
                data_dir: Some("~/data".into()),
                cache_dir: Some("$HOME/cache".into()),
            },
        };

        let data = data_dir(&config, None);
        let cache = cache_dir(&config, None);

        // Both should be expanded but different
        assert!(!data.to_string_lossy().contains("~"));
        assert!(!cache.to_string_lossy().contains("$HOME"));
        assert_ne!(data, cache);

        // Both should end with expected paths
        assert!(data.to_string_lossy().ends_with("/data"));
        assert!(cache.to_string_lossy().ends_with("/cache"));
    }

    #[test]
    fn test_config_relative_paths_unchanged() {
        let config = Config {
            server: schema::ServerConfig::default(),
            storage: schema::StorageConfig {
                data_dir: Some("./relative/data".into()),
                cache_dir: Some("relative/cache".into()),
            },
        };

        let data = data_dir(&config, None);
        let cache = cache_dir(&config, None);

        // Relative paths should remain unchanged
        assert_eq!(data, PathBuf::from("./relative/data"));
        assert_eq!(cache, PathBuf::from("relative/cache"));
    }

    #[test]
    fn test_config_complex_expansion_patterns() {
        let config = Config {
            server: schema::ServerConfig::default(),
            storage: schema::StorageConfig {
                data_dir: Some("$HOME/.local/share/app/data".into()),
                cache_dir: Some("~/Library/Caches/app".into()),
            },
        };

        let data = data_dir(&config, None);
        let cache = cache_dir(&config, None);

        // Should handle complex nested paths
        assert!(!data.to_string_lossy().contains("$HOME"));
        assert!(!cache.to_string_lossy().contains("~"));

        assert!(data.to_string_lossy().contains(".local/share/app/data"));
        assert!(cache.to_string_lossy().contains("Library/Caches/app"));
    }

    #[test]
    fn test_config_with_unicode_paths() {
        let config = Config {
            server: schema::ServerConfig::default(),
            storage: schema::StorageConfig {
                data_dir: Some("~/Documents/测试应用/数据".into()),
                cache_dir: Some("$HOME/Cache/äöü-app".into()),
            },
        };

        let data = data_dir(&config, None);
        let cache = cache_dir(&config, None);

        // Should handle unicode correctly
        assert!(!data.to_string_lossy().contains("~"));
        assert!(!cache.to_string_lossy().contains("$HOME"));

        assert!(data.to_string_lossy().contains("测试应用"));
        assert!(cache.to_string_lossy().contains("äöü-app"));
    }

    #[test]
    fn test_expand_optional_path_with_config() {
        let config_path = Some(PathBuf::from("~/test/path"));
        let default_path = || PathBuf::from("/default/path");

        let result = expand_optional_path(config_path.as_ref(), None, default_path);

        // Should expand the config path, not use default
        assert!(!result.to_string_lossy().contains("~"));
        assert!(result.to_string_lossy().ends_with("/test/path"));
        assert!(!result.to_string_lossy().contains("/default/path"));
    }

    #[test]
    fn test_expand_optional_path_with_none() {
        let config_path: Option<PathBuf> = None;
        let default_path = || PathBuf::from("/default/path");

        let result = expand_optional_path(config_path.as_ref(), None, default_path);

        // Should use default path
        assert_eq!(result, PathBuf::from("/default/path"));
    }

    #[test]
    fn test_expand_optional_path_with_dollar_home() {
        let config_path = Some(PathBuf::from("$HOME/config"));
        let default_path = || PathBuf::from("/default");

        let result = expand_optional_path(config_path.as_ref(), None, default_path);

        // Should expand $HOME
        assert!(!result.to_string_lossy().contains("$HOME"));
        assert!(result.to_string_lossy().ends_with("/config"));
    }

    #[test]
    fn test_expand_required_path() {
        let path = "~/required/path";
        let result = expand_required_path(path, None);

        // Should expand the tilde
        assert!(!result.to_string_lossy().contains("~"));
        assert!(result.to_string_lossy().ends_with("/required/path"));
    }

    #[test]
    fn test_expand_required_path_with_pathbuf() {
        let path = PathBuf::from("$HOME/another/path");
        let result = expand_required_path(&path, None);

        // Should expand $HOME
        assert!(!result.to_string_lossy().contains("$HOME"));
        assert!(result.to_string_lossy().ends_with("/another/path"));
    }

    #[test]
    fn test_expand_required_path_absolute() {
        let path = "/absolute/path";
        let result = expand_required_path(path, None);

        // Should remain unchanged
        assert_eq!(result, PathBuf::from("/absolute/path"));
    }

    #[test]
    fn test_config_with_nonexistent_config_dir() {
        use super::super::schema::*;

        let config = Config {
            server: ServerConfig::default(),
            storage: StorageConfig {
                data_dir: Some("./relative/to/config".into()),
                cache_dir: Some("../another/relative".into()),
            },
        };

        // Test with nonexistent config directory
        let nonexistent_dir = std::path::Path::new("/this/directory/does/not/exist");

        let data = data_dir(&config, Some(nonexistent_dir));
        let cache = cache_dir(&config, Some(nonexistent_dir));

        // Should still work and create meaningful paths
        assert!(data.to_string_lossy().contains("relative/to/config"));
        assert!(cache.to_string_lossy().contains("another/relative"));

        // Paths should be resolved relative to the nonexistent config dir
        assert!(data.is_absolute());
        assert!(cache.is_absolute());
    }

    #[test]
    fn test_path_expansion_with_empty_strings() {
        let empty_path = PathBuf::from("");
        let result = expand_required_path(empty_path, None);

        // Should handle empty paths gracefully
        assert_eq!(result, PathBuf::from(""));

        // Test with optional path
        let result =
            expand_optional_path(Some(&PathBuf::from("")), None, || PathBuf::from("/default"));

        // Empty path should still use the provided path, not default
        assert_eq!(result, PathBuf::from(""));
    }

    #[test]
    fn test_very_long_config_paths() {
        use super::super::schema::*;

        // Create a very long path (within filesystem limits)
        let long_component = "very_long_directory_name_that_tests_path_handling".repeat(5);
        let long_path = format!("~/{}", long_component);

        let config = Config {
            server: ServerConfig::default(),
            storage: StorageConfig {
                data_dir: Some(long_path.clone().into()),
                cache_dir: Some(long_path.into()),
            },
        };

        let data = data_dir(&config, None);
        let cache = cache_dir(&config, None);

        // Should handle long paths without issues
        assert!(data.is_absolute());
        assert!(cache.is_absolute());
        assert!(!data.to_string_lossy().contains("~"));
        assert!(!cache.to_string_lossy().contains("~"));
    }
}
