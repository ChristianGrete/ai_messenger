use anyhow::Result;
use clap::{Arg, ArgAction, ArgMatches, Command};
use std::path::{Path, PathBuf};

/// Create a generic path command (for cache, data, etc.)
pub fn create_path_command(name: &'static str, about: &'static str) -> Command {
    let cmd = Command::new(name)
        .about(about)
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
            Arg::new("verbose")
                .long("verbose")
                .short('V')
                .help("Enable verbose output (sets log-level to debug)")
                .action(ArgAction::SetTrue),
        );

    // Apply consistent help styling
    crate::cli::options::help::apply(cmd)
}

/// Generic run function for path commands
pub async fn run_path_command<F>(matches: &ArgMatches, path_fn: F) -> Result<()>
where
    F: Fn(&crate::config::Config, Option<&Path>) -> PathBuf,
{
    let config_file = matches.get_one::<String>("config").cloned();
    let log_level = crate::cli::options::logging::extract_log_level(matches);

    // Initialize logging with the requested level
    if let Err(e) = crate::utils::init_logging(&log_level) {
        eprintln!("Failed to initialize logging: {}", e);
        // Continue without logging rather than fail
    }

    // Load configuration using same logic as serve (but silent for non-debug)
    let (config, config_dir) = if log_level == "debug" {
        crate::config::load_config(config_file)?
    } else {
        crate::config::load_config_silent(config_file)?
    };

    // Get the path using the provided function and print it
    let path = path_fn(&config, config_dir.as_deref());
    println!("{}", path.display());

    Ok(())
}

#[cfg(test)]
pub mod test_utils {
    use super::*;

    /// Test helper for path commands
    pub fn test_path_command(name: &'static str, about: &'static str) {
        let cmd = create_path_command(name, about);

        // Basic structure tests
        assert_eq!(cmd.get_name(), name);
        let about_str = format!("{}", cmd.get_about().unwrap());
        assert_eq!(about_str, about);
        assert!(cmd.is_disable_help_flag_set());
        assert_eq!(cmd.get_arguments().count(), 4); // config, help, log-level, verbose

        // Should have all expected arguments
        assert!(cmd.get_arguments().any(|arg| arg.get_id() == "config"));
        assert!(cmd.get_arguments().any(|arg| arg.get_id() == "help"));
        assert!(cmd.get_arguments().any(|arg| arg.get_id() == "log-level"));
        assert!(cmd.get_arguments().any(|arg| arg.get_id() == "verbose"));
    }

    /// Test helper for argument properties
    pub fn test_path_command_args(cmd: &Command) {
        // Test config argument
        let config_arg = cmd
            .get_arguments()
            .find(|arg| arg.get_id() == "config")
            .unwrap();
        assert!(!config_arg.is_required_set());
        assert_eq!(config_arg.get_num_args(), Some(1.into()));
        assert_eq!(
            config_arg.get_value_names(),
            Some(vec!["FILE".into()].as_slice())
        );

        // Test help argument
        let help_arg = cmd
            .get_arguments()
            .find(|arg| arg.get_id() == "help")
            .unwrap();
        assert!(matches!(help_arg.get_action(), ArgAction::Help));
        assert!(help_arg.get_short() == Some('h'));
        assert!(help_arg.get_long() == Some("help"));

        // Test log-level argument
        let log_level_arg = cmd
            .get_arguments()
            .find(|arg| arg.get_id() == "log-level")
            .unwrap();
        assert_eq!(log_level_arg.get_num_args(), Some(1.into()));
        assert!(log_level_arg.get_short() == Some('l'));
        assert!(log_level_arg.get_long() == Some("log-level"));

        // Test verbose argument
        let verbose_arg = cmd
            .get_arguments()
            .find(|arg| arg.get_id() == "verbose")
            .unwrap();
        assert!(matches!(verbose_arg.get_action(), ArgAction::SetTrue));
        assert!(verbose_arg.get_short() == Some('V'));
        assert!(verbose_arg.get_long() == Some("verbose"));
    }

    /// Test helper for parsing
    pub fn test_path_command_parsing(name: &'static str) {
        let cmd = create_path_command(name, "Test command");

        // Test parsing with no args
        let matches = cmd.try_get_matches_from([name]).unwrap();
        assert!(matches.get_one::<String>("config").is_none());

        // Test parsing with config
        let cmd = create_path_command(name, "Test command");
        let matches = cmd
            .try_get_matches_from([name, "--config", "/path/to/config.toml"])
            .unwrap();
        assert_eq!(
            matches.get_one::<String>("config"),
            Some(&"/path/to/config.toml".to_string())
        );
    }

    /// Test helper for help flags
    pub fn test_path_command_help_flags(name: &'static str) {
        // Test long help flag
        let cmd = create_path_command(name, "Test command");
        let result = cmd.try_get_matches_from([name, "--help"]);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.kind(), clap::error::ErrorKind::DisplayHelp);

        // Test short help flag
        let cmd = create_path_command(name, "Test command");
        let result = cmd.try_get_matches_from([name, "-h"]);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.kind(), clap::error::ErrorKind::DisplayHelp);
    }

    #[test]
    fn test_create_path_command_structure() {
        let cmd = create_path_command("test", "Test command");

        // Basic properties
        assert_eq!(cmd.get_name(), "test");
        assert_eq!(cmd.get_about().unwrap().to_string(), "Test command");
        assert!(cmd.is_disable_help_flag_set());

        // Should have exactly 4 arguments
        assert_eq!(cmd.get_arguments().count(), 4);

        // Verify all required arguments exist
        let arg_names: Vec<_> = cmd
            .get_arguments()
            .map(|arg| arg.get_id().as_str())
            .collect();
        assert!(arg_names.contains(&"config"));
        assert!(arg_names.contains(&"help"));
        assert!(arg_names.contains(&"log-level"));
        assert!(arg_names.contains(&"verbose"));
    }

    #[test]
    fn test_create_path_command_log_level_validation() {
        let cmd = create_path_command("test", "Test command");

        // Valid log level should work
        let result = cmd.try_get_matches_from(["test", "--log-level", "debug"]);
        assert!(result.is_ok());

        // Invalid log level should fail
        let cmd = create_path_command("test", "Test command");
        let result = cmd.try_get_matches_from(["test", "--log-level", "invalid"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_path_command_verbose_flag() {
        let cmd = create_path_command("test", "Test command");
        let matches = cmd.try_get_matches_from(["test", "--verbose"]).unwrap();

        assert!(matches.get_flag("verbose"));

        // Should use verbose precedence for log level
        let log_level = crate::cli::options::logging::extract_log_level(&matches);
        assert_eq!(log_level, "debug");
    }
}
