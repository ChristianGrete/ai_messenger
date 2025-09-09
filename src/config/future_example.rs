// This is a demonstration of how the path expansion system can be
// extended for future configuration options.
//
// This file is not part of the main codebase yet, but shows the pattern
// for adding new path-based configuration options.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// Example: Future logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Optional custom log file path
    /// Supports ~ and $HOME expansion
    pub log_file: Option<PathBuf>,

    /// Optional custom error log path
    /// Supports ~ and $HOME expansion
    pub error_log: Option<PathBuf>,

    /// Log level (info, warn, error, debug)
    #[serde(default = "default_log_level")]
    pub level: String,
}

// Example: Future AI providers configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AiProvidersConfig {
    /// Directory for caching AI models
    /// Supports ~ and $HOME expansion
    pub models_cache: Option<PathBuf>,

    /// Directory for provider config templates
    /// Supports ~ and $HOME expansion
    pub config_templates: Option<PathBuf>,
}

// Example: Future security configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecurityConfig {
    /// Path to private key file
    /// Supports ~ and $HOME expansion
    pub private_key: Option<PathBuf>,

    /// Directory containing certificates
    /// Supports ~ and $HOME expansion
    pub certificates_dir: Option<PathBuf>,
}

// Example: Extended main config (future version)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FutureConfig {
    #[serde(default)]
    pub server: crate::config::schema::ServerConfig,

    #[serde(default)]
    pub storage: crate::config::schema::StorageConfig,

    // Future config sections:
    #[serde(default)]
    pub logging: LoggingConfig,

    #[serde(default)]
    pub ai_providers: AiProvidersConfig,

    #[serde(default)]
    pub security: SecurityConfig,
}

// Example implementations using the generic helpers
#[allow(dead_code)]
impl FutureConfig {
    /// Get effective log file path with expansion
    pub fn log_file_path(&self, config_dir: Option<&std::path::Path>) -> PathBuf {
        crate::config::expand_optional_path(self.logging.log_file.as_ref(), config_dir, || {
            default_log_file_path()
        })
    }

    /// Get effective error log path with expansion
    pub fn error_log_path(&self, config_dir: Option<&std::path::Path>) -> PathBuf {
        crate::config::expand_optional_path(self.logging.error_log.as_ref(), config_dir, || {
            default_error_log_path()
        })
    }

    /// Get effective models cache directory with expansion
    pub fn models_cache_dir(&self, config_dir: Option<&std::path::Path>) -> PathBuf {
        crate::config::expand_optional_path(
            self.ai_providers.models_cache.as_ref(),
            config_dir,
            default_models_cache_dir,
        )
    }

    /// Get private key path with expansion (if configured)
    pub fn private_key_path(&self, config_dir: Option<&std::path::Path>) -> Option<PathBuf> {
        self.security
            .private_key
            .as_ref()
            .map(|path| crate::config::expand_required_path(path, config_dir))
    }
}

// Default implementations
impl Default for LoggingConfig {
    fn default() -> Self {
        LoggingConfig {
            log_file: None,
            error_log: None,
            level: default_log_level(),
        }
    }
}

// Default value functions
#[allow(dead_code)]
fn default_log_level() -> String {
    "info".to_string()
}

#[allow(dead_code)]
fn default_log_file_path() -> PathBuf {
    if let Some(data_dir) = dirs::data_dir() {
        data_dir
            .join("com.christiangrete.ai_messenger")
            .join("app.log")
    } else {
        PathBuf::from("./app.log")
    }
}

#[allow(dead_code)]
fn default_error_log_path() -> PathBuf {
    if let Some(data_dir) = dirs::data_dir() {
        data_dir
            .join("com.christiangrete.ai_messenger")
            .join("error.log")
    } else {
        PathBuf::from("./error.log")
    }
}

#[allow(dead_code)]
fn default_models_cache_dir() -> PathBuf {
    if let Some(cache_dir) = dirs::cache_dir() {
        cache_dir
            .join("com.christiangrete.ai_messenger")
            .join("models")
    } else {
        PathBuf::from("./models")
    }
}

// Example TOML configuration that would work with this:
//
// [server]
// host = "127.0.0.1"
// port = 8080
//
// [storage]
// data_dir = "~/my_app/data"
// cache_dir = "$HOME/.cache/my_app"
//
// [logging]
// log_file = "~/logs/app.log"
// error_log = "$HOME/logs/errors.log"
// level = "debug"
//
// [ai_providers]
// models_cache = "~/ai_models"
// config_templates = "$HOME/.ai_configs"
//
// [security]
// private_key = "~/.ssh/app_key"
// certificates_dir = "$HOME/certs"

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_future_config_path_expansion() {
        let config = FutureConfig {
            server: Default::default(),
            storage: Default::default(),
            logging: LoggingConfig {
                log_file: Some("~/logs/app.log".into()),
                error_log: Some("$HOME/logs/error.log".into()),
                level: "debug".to_string(),
            },
            ai_providers: AiProvidersConfig {
                models_cache: Some("~/ai_models".into()),
                config_templates: None,
            },
            security: SecurityConfig {
                private_key: Some("~/.ssh/key".into()),
                certificates_dir: None,
            },
        };

        // Test path expansion (no config_dir for this test)
        let log_path = config.log_file_path(None);
        let error_path = config.error_log_path(None);
        let models_path = config.models_cache_dir(None);
        let key_path = config.private_key_path(None).unwrap();

        // All should be expanded
        assert!(!log_path.to_string_lossy().contains("~"));
        assert!(!error_path.to_string_lossy().contains("$HOME"));
        assert!(!models_path.to_string_lossy().contains("~"));
        assert!(!key_path.to_string_lossy().contains("~"));

        // Should contain expected parts
        assert!(log_path.to_string_lossy().ends_with("/logs/app.log"));
        assert!(error_path.to_string_lossy().ends_with("/logs/error.log"));
        assert!(models_path.to_string_lossy().ends_with("/ai_models"));
        assert!(key_path.to_string_lossy().ends_with("/.ssh/key"));
    }

    #[test]
    fn test_toml_parsing() {
        let toml_content = r#"
[logging]
log_file = "~/custom.log"
level = "warn"

[ai_providers]
models_cache = "$HOME/models"
"#;

        let config: FutureConfig = toml::from_str(toml_content).unwrap();

        assert_eq!(config.logging.log_file, Some("~/custom.log".into()));
        assert_eq!(config.logging.level, "warn");
        assert_eq!(
            config.ai_providers.models_cache,
            Some("$HOME/models".into())
        );
    }
}
