use std::path::PathBuf;

const APP_DOMAIN: &str = "com.christiangrete.ai_messenger";

/// Default server host for all contexts
pub const DEFAULT_SERVER_HOST: &str = "127.0.0.1";

/// Default server port for all contexts
pub const DEFAULT_SERVER_PORT: u16 = 8080;

/// Default server port as string for CLI (clap needs &str)
pub const DEFAULT_SERVER_PORT_STR: &str = "8080";
/// Default server base path for all contexts
pub const DEFAULT_SERVER_BASE_PATH: &str = "";

/// Get default server host as String (for serde defaults)
pub fn default_host() -> String {
    DEFAULT_SERVER_HOST.to_string()
}

/// Get default server port (for serde defaults)
pub fn default_port() -> u16 {
    DEFAULT_SERVER_PORT
}

/// Get default server base path as String (for serde defaults)
pub fn default_base_path() -> String {
    DEFAULT_SERVER_BASE_PATH.to_string()
}

/// Generic helper for platform-specific directories with fallbacks
fn platform_dir_with_fallback<F>(dir_fn: F, local_fallback: &str) -> PathBuf
where
    F: FnOnce() -> Option<PathBuf>,
{
    if let Some(platform_dir) = dir_fn() {
        platform_dir.join(APP_DOMAIN)
    } else {
        PathBuf::from(local_fallback)
    }
}

/// Get the default data directory using platform-specific paths
pub fn default_data_dir() -> PathBuf {
    platform_dir_with_fallback(dirs::data_dir, "./ai_messenger")
}

/// Get the default cache directory using platform-specific paths
pub fn default_cache_dir() -> PathBuf {
    platform_dir_with_fallback(dirs::cache_dir, "./ai_messenger/cache")
}

/// Get the default config directory using platform-specific paths
pub fn default_config_dir() -> PathBuf {
    dirs::config_dir().unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
}

/// Get the platform-specific config file path
pub fn platform_config_file() -> PathBuf {
    default_config_dir().join(format!("{}.toml", APP_DOMAIN))
}

/// Get the home directory dotfile config path
pub fn home_config_file() -> PathBuf {
    if let Some(home_dir) = dirs::home_dir() {
        home_dir.join(".ai_messenger.toml")
    } else {
        PathBuf::from("./.ai_messenger.toml")
    }
}

/// Get the local config file path
pub fn local_config_file() -> PathBuf {
    PathBuf::from("./ai_messenger.toml")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_data_dir() {
        let data_dir = default_data_dir();

        // Should contain our app domain
        assert!(data_dir.to_string_lossy().contains(APP_DOMAIN));

        // Should be an absolute path or local fallback
        assert!(data_dir.is_absolute() || data_dir.starts_with("./ai_messenger"));
    }

    #[test]
    fn test_default_cache_dir() {
        let cache_dir = default_cache_dir();

        // Should contain our app domain or be local fallback
        let path_str = cache_dir.to_string_lossy();
        assert!(path_str.contains(APP_DOMAIN) || path_str.contains("./ai_messenger/cache"));

        // Should be an absolute path or local fallback
        assert!(cache_dir.is_absolute() || cache_dir.starts_with("./ai_messenger"));
    }

    #[test]
    fn test_default_config_dir() {
        let config_dir = default_config_dir();

        // Should be absolute or fallback to current dir
        assert!(config_dir.is_absolute() || config_dir == PathBuf::from("."));
    }

    #[test]
    fn test_platform_config_file() {
        let config_file = platform_config_file();

        // Should end with our domain.toml
        assert!(
            config_file
                .to_string_lossy()
                .ends_with(&format!("{}.toml", APP_DOMAIN))
        );
    }

    #[test]
    fn test_home_config_file() {
        let config_file = home_config_file();

        // Should end with .ai_messenger.toml
        assert!(
            config_file
                .to_string_lossy()
                .ends_with(".ai_messenger.toml")
        );
    }

    #[test]
    fn test_local_config_file() {
        let config_file = local_config_file();

        assert_eq!(config_file, PathBuf::from("./ai_messenger.toml"));
    }

    #[test]
    fn test_app_domain_constant() {
        assert_eq!(APP_DOMAIN, "com.christiangrete.ai_messenger");
    }

    #[test]
    fn test_config_file_precedence() {
        let local = local_config_file();
        let home = home_config_file();
        let platform = platform_config_file();

        // All should be different paths
        assert_ne!(local, home);
        assert_ne!(local, platform);
        assert_ne!(home, platform);

        // Local should be most specific (current directory)
        assert!(local.to_string_lossy().starts_with("./"));

        // Home should be dotfile
        assert!(home.file_name().unwrap().to_string_lossy().starts_with("."));

        // Platform should be in config directory
        assert!(
            platform
                .file_name()
                .unwrap()
                .to_string_lossy()
                .contains(APP_DOMAIN)
        );
    }
}
