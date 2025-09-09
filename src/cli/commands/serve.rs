use crate::config::defaults::{DEFAULT_SERVER_HOST, DEFAULT_SERVER_PORT, DEFAULT_SERVER_PORT_STR};
use anyhow::Result;
use clap::parser::ValueSource;
use clap::{Arg, ArgAction, ArgMatches, Command};

pub fn command() -> Command {
    let cmd = Command::new("serve")
        .about("Start the ai_messenger service")
        .disable_help_flag(true)
        .arg(
            Arg::new("config")
                .long("config")
                .value_name("FILE")
                .help("Path to configuration file")
                .num_args(1),
        )
        .arg(
            Arg::new("help")
                .long("help")
                .short('h')
                .help("Print help")
                .action(ArgAction::Help),
        )
        .arg(
            Arg::new("host")
                .long("host")
                .value_name("HOST")
                .help(format!("Bind host (default: {})", DEFAULT_SERVER_HOST))
                .default_value(DEFAULT_SERVER_HOST)
                .num_args(1),
        )
        .arg(
            Arg::new("log-level")
                .long("log-level")
                .short('l')
                .value_name("LEVEL")
                .help("Set the logging level")
                .value_parser(crate::cli::options::logging::LOG_LEVEL_VALUES)
                .default_value(crate::cli::options::logging::DEFAULT_LOG_LEVEL)
                .num_args(1),
        )
        .arg(
            Arg::new("port")
                .long("port")
                .value_name("PORT")
                .help(format!("Bind port (default: {})", DEFAULT_SERVER_PORT))
                .default_value(DEFAULT_SERVER_PORT_STR)
                .num_args(1),
        )
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .short('V')
                .help("Enable verbose output (sets log-level to debug)")
                .action(ArgAction::SetTrue),
        );

    // Apply consistent help styling
    crate::cli::options::help::apply(cmd)
}

pub async fn run(m: &ArgMatches) -> Result<()> {
    let serve_config = extract_config(m);

    // Initialize logging as early as possible
    if let Err(e) = crate::utils::init_logging(&serve_config.log_level) {
        eprintln!("Failed to initialize logging: {}", e);
        // Continue without logging rather than fail
    }

    tracing::info!("Starting ai_messenger server");
    tracing::debug!("Log level set to: {}", serve_config.log_level);

    // Load configuration
    let (config, config_dir) = crate::config::load_config(serve_config.config_file.clone())?;

    // Use config values, with CLI overrides taking precedence
    let host = serve_config.host;
    let port = serve_config.port;
    let log_level = serve_config.log_level;

    tracing::info!("Server will bind to {}:{}", host, port);

    // Start the server (server will handle its own logging based on log_level)
    let startup_config = crate::server::startup::ServerStartupConfig {
        config,
        config_dir,
        host,
        log_level,
        port,
    };
    crate::server::start(startup_config).await?;

    Ok(())
}

#[derive(Debug)]
pub struct ServeConfig {
    pub config_file: Option<String>,
    pub host: String,
    pub log_level: String,
    pub port: u16,
}

/// Extract configuration from CLI arguments with proper precedence:
/// CLI explicit > Config file > Default values
fn extract_config(matches: &ArgMatches) -> ServeConfig {
    // Load config file first to get potential values
    let config_file = matches.get_one::<String>("config").cloned();
    let (config, _) = crate::config::load_config_silent(config_file.clone()).unwrap_or_default();

    let log_level = crate::cli::options::logging::extract_log_level(matches);

    // Host precedence: CLI explicit > Config file > Default
    let host = match matches.value_source("host") {
        Some(ValueSource::CommandLine) => {
            // User explicitly set --host
            matches.get_one::<String>("host").unwrap().clone()
        }
        _ => {
            // Use config file value, which already has built-in default
            config.server.host.clone()
        }
    };

    // Port precedence: CLI explicit > Config file > Default
    let port = match matches.value_source("port") {
        Some(ValueSource::CommandLine) => {
            // User explicitly set --port
            matches
                .get_one::<String>("port")
                .unwrap()
                .parse()
                .unwrap_or(DEFAULT_SERVER_PORT)
        }
        _ => {
            // Use config file value, which already has built-in default
            config.server.port
        }
    };

    ServeConfig {
        config_file,
        host,
        log_level,
        port,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_creation() {
        let cmd = command();

        assert_eq!(cmd.get_name(), "serve");
        // Note: get_about() returns StyledStr in clap 4.5, so we test differently
        let about_str = format!("{}", cmd.get_about().unwrap());
        assert_eq!(about_str, "Start the ai_messenger service");
    }

    #[test]
    fn test_command_has_required_args() {
        let cmd = command();

        // Should have all expected arguments
        assert!(cmd.get_arguments().any(|arg| arg.get_id() == "config"));
        assert!(cmd.get_arguments().any(|arg| arg.get_id() == "help"));
        assert!(cmd.get_arguments().any(|arg| arg.get_id() == "host"));
        assert!(cmd.get_arguments().any(|arg| arg.get_id() == "log-level"));
        assert!(cmd.get_arguments().any(|arg| arg.get_id() == "port"));
        assert!(cmd.get_arguments().any(|arg| arg.get_id() == "verbose"));
    }

    #[test]
    fn test_command_defaults() {
        let cmd = command();

        let host_arg = cmd
            .get_arguments()
            .find(|arg| arg.get_id() == "host")
            .unwrap();
        let port_arg = cmd
            .get_arguments()
            .find(|arg| arg.get_id() == "port")
            .unwrap();

        // Check default values exist (comparing OsStr values properly)
        assert!(!host_arg.get_default_values().is_empty());
        assert!(!port_arg.get_default_values().is_empty());

        // Check actual default values by converting to string
        let host_default = host_arg.get_default_values()[0].to_string_lossy();
        let port_default = port_arg.get_default_values()[0].to_string_lossy();
        assert_eq!(host_default, DEFAULT_SERVER_HOST);
        assert_eq!(port_default, DEFAULT_SERVER_PORT_STR);
    }

    #[test]
    fn test_extract_config_defaults() {
        let cmd = command();
        let matches = cmd.try_get_matches_from(["serve"]).unwrap();

        let config = extract_config(&matches);

        assert_eq!(config.config_file, None);
        assert_eq!(config.host, DEFAULT_SERVER_HOST);
        assert_eq!(config.log_level, "info");
        assert_eq!(config.port, DEFAULT_SERVER_PORT);
    }

    #[test]
    fn test_extract_config_with_custom_values() {
        let cmd = command();
        let matches = cmd
            .try_get_matches_from([
                "serve",
                "--host",
                "0.0.0.0",
                "--port",
                "3000",
                "--config",
                "/custom/config.toml",
                "--log-level",
                "debug",
            ])
            .unwrap();

        let config = extract_config(&matches);

        assert_eq!(config.config_file, Some("/custom/config.toml".to_string()));
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.log_level, "debug");
        assert_eq!(config.port, 3000);
    }

    #[test]
    fn test_extract_config_invalid_port() {
        let cmd = command();
        let matches = cmd
            .try_get_matches_from(["serve", "--port", "invalid"])
            .unwrap();

        let config = extract_config(&matches);

        // Should fallback to DEFAULT_SERVER_PORT for invalid port
        assert_eq!(config.port, DEFAULT_SERVER_PORT);
    }

    #[test]
    fn test_extract_config_verbose_flag() {
        let cmd = command();
        let matches = cmd.try_get_matches_from(["serve", "--verbose"]).unwrap();

        let config = extract_config(&matches);

        assert_eq!(config.log_level, "debug"); // --verbose sets log-level to debug
    }

    #[test]
    fn test_extract_config_log_level() {
        let cmd = command();
        let matches = cmd
            .try_get_matches_from(["serve", "--log-level", "warn"])
            .unwrap();

        let config = extract_config(&matches);

        assert_eq!(config.log_level, "warn");
    }

    #[test]
    fn test_extract_config_verbose_overrides_log_level() {
        let cmd = command();
        let matches = cmd
            .try_get_matches_from(["serve", "--log-level", "warn", "--verbose"])
            .unwrap();

        let config = extract_config(&matches);

        assert_eq!(config.log_level, "debug"); // --verbose overrides --log-level
    }

    #[test]
    fn test_host_precedence_cli_over_config() {
        // Test that explicit CLI --host overrides config file
        let cmd = command();
        let matches = cmd
            .try_get_matches_from(["serve", "--host", "192.168.1.100"])
            .unwrap();
        let config = extract_config(&matches);
        assert_eq!(config.host, "192.168.1.100"); // CLI explicit wins
    }

    #[test]
    fn test_port_precedence_cli_over_config() {
        // Test that explicit CLI --port overrides config file
        let cmd = command();
        let matches = cmd
            .try_get_matches_from(["serve", "--port", "9999"])
            .unwrap();
        let config = extract_config(&matches);
        assert_eq!(config.port, 9999); // CLI explicit wins
    }

    #[test]
    fn test_host_fallback_to_default_when_no_cli() {
        // Test that when no CLI host is provided, we fall back to config/default
        let cmd = command();
        let matches = cmd.try_get_matches_from(["serve"]).unwrap();
        let config = extract_config(&matches);
        // Should use config file value or default (127.0.0.1)
        assert_eq!(config.host, "127.0.0.1"); // Config default
    }

    #[test]
    fn test_port_fallback_to_default_when_no_cli() {
        // Test that when no CLI port is provided, we fall back to config/default
        let cmd = command();
        let matches = cmd.try_get_matches_from(["serve"]).unwrap();
        let config = extract_config(&matches);
        // Should use config file value or default (8080)
        assert_eq!(config.port, 8080); // Config default
    }

    #[test]
    fn test_serve_config_debug() {
        let config = ServeConfig {
            config_file: Some("test.toml".to_string()),
            host: "localhost".to_string(),
            log_level: "debug".to_string(),
            port: DEFAULT_SERVER_PORT,
        };

        // Should be debuggable
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("debug"));
        assert!(debug_str.contains("localhost"));
        assert!(debug_str.contains("test.toml"));
        assert!(debug_str.contains(&DEFAULT_SERVER_PORT.to_string()));
    }

    #[test]
    fn test_constants() {
        assert_eq!(DEFAULT_SERVER_HOST, "127.0.0.1");
        assert_eq!(DEFAULT_SERVER_PORT, 8080);
    }

    #[test]
    fn test_argument_properties() {
        let cmd = command();

        // Test verbose argument is a flag
        let verbose_arg = cmd
            .get_arguments()
            .find(|arg| arg.get_id() == "verbose")
            .unwrap();
        assert!(matches!(verbose_arg.get_action(), clap::ArgAction::SetTrue));

        // Test log-level argument accepts one value
        let log_level_arg = cmd
            .get_arguments()
            .find(|arg| arg.get_id() == "log-level")
            .unwrap();
        assert_eq!(log_level_arg.get_num_args(), Some(1.into()));
    }

    #[test]
    fn test_precedence_with_config_file() {
        // This test simulates the scenario where we have both config file AND CLI args
        // and verifies CLI wins over config file values
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_precedence.toml");

        // Create a config file with specific values
        let config_content = r#"
[server]
host = "192.168.1.1"
port = 9000
"#;
        fs::write(&config_path, config_content).unwrap();

        let cmd = command();
        let matches = cmd
            .try_get_matches_from([
                "serve",
                "--config",
                &config_path.to_string_lossy(),
                "--host",
                "127.0.0.1",
                "--port",
                "8080",
            ])
            .unwrap();

        let config = extract_config(&matches);

        // CLI values should override config file values
        assert_eq!(config.host, "127.0.0.1"); // CLI override
        assert_eq!(config.port, 8080); // CLI override
    }

    #[test]
    fn test_precedence_config_file_over_defaults() {
        // Test that config file values are used when no CLI args are provided
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config_defaults.toml");

        // Create a config file with custom values
        let config_content = r#"
[server]
host = "0.0.0.0"
port = 3000
"#;
        fs::write(&config_path, config_content).unwrap();

        let cmd = command();
        let matches = cmd
            .try_get_matches_from(["serve", "--config", &config_path.to_string_lossy()])
            .unwrap();

        let config = extract_config(&matches);

        // Config file values should be used (not defaults)
        assert_eq!(config.host, "0.0.0.0"); // From config file
        assert_eq!(config.port, 3000); // From config file
    }

    #[test]
    fn test_precedence_defaults_when_no_config() {
        // Test that defaults are used when no CLI args AND no config file
        let cmd = command();
        let matches = cmd.try_get_matches_from(["serve"]).unwrap();

        let config = extract_config(&matches);

        // Should get defaults since no CLI args and no config file
        assert_eq!(config.host, DEFAULT_SERVER_HOST);
        assert_eq!(config.port, DEFAULT_SERVER_PORT);
    }

    #[test]
    fn test_precedence_partial_config_file() {
        // Test mixed scenario: config file has only some values
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_partial.toml");

        // Config file only has host, no port
        let config_content = r#"
[server]
host = "0.0.0.0"
# port is missing - should use default
"#;
        fs::write(&config_path, config_content).unwrap();

        let cmd = command();
        let matches = cmd
            .try_get_matches_from(["serve", "--config", &config_path.to_string_lossy()])
            .unwrap();

        let config = extract_config(&matches);

        // Host from config, port from default
        assert_eq!(config.host, "0.0.0.0"); // From config
        assert_eq!(config.port, DEFAULT_SERVER_PORT); // Serde default
    }

    #[test]
    fn test_precedence_invalid_config_file() {
        // Test that we fall back to defaults when config file is invalid
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("invalid.toml");

        // Create invalid TOML
        let config_content = r#"
[server
host = "broken
"#;
        fs::write(&config_path, config_content).unwrap();

        let cmd = command();
        let matches = cmd
            .try_get_matches_from(["serve", "--config", &config_path.to_string_lossy()])
            .unwrap();

        let config = extract_config(&matches);

        // Should fallback to defaults when config is invalid
        assert_eq!(config.host, DEFAULT_SERVER_HOST);
        assert_eq!(config.port, DEFAULT_SERVER_PORT);
    }

    #[test]
    fn test_precedence_nonexistent_config_file() {
        // Test that we fall back to defaults when specified config file doesn't exist
        let cmd = command();
        let matches = cmd
            .try_get_matches_from(["serve", "--config", "/nonexistent/path/config.toml"])
            .unwrap();

        let config = extract_config(&matches);

        // Should fallback to defaults when config file doesn't exist
        assert_eq!(config.host, DEFAULT_SERVER_HOST);
        assert_eq!(config.port, DEFAULT_SERVER_PORT);
    }

    #[tokio::test]
    async fn test_run_function_exists() {
        let cmd = command();
        let matches = cmd.try_get_matches_from(["serve"]).unwrap();

        // NOTE: This test only verifies the function signature exists
        // Actually calling run() would start a real server, which is not appropriate for unit tests
        // Integration tests should be used for testing the actual server startup

        // Test that we can extract config without errors
        let config = extract_config(&matches);
        assert_eq!(config.host, DEFAULT_SERVER_HOST);
        assert_eq!(config.port, DEFAULT_SERVER_PORT);
    }
}
