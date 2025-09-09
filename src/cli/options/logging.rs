use clap::ArgMatches;

/// Default log level for all commands
pub const DEFAULT_LOG_LEVEL: &str = "info";

/// Valid log level values for all commands (aligned with tracing levels)
pub const LOG_LEVEL_VALUES: [&str; 6] = ["trace", "debug", "info", "warn", "error", "off"];

/// Extract log level from matches, with --verbose override
pub fn extract_log_level(matches: &ArgMatches) -> String {
    if matches.get_flag("verbose") {
        "debug".to_string()
    } else {
        matches.get_one::<String>("log-level").unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Arg, ArgAction, Command};

    fn create_test_command() -> Command {
        Command::new("test")
            .arg(
                Arg::new("log-level")
                    .long("log-level")
                    .short('l')
                    .value_name("LEVEL")
                    .help("Set the logging level")
                    .value_parser(LOG_LEVEL_VALUES)
                    .default_value(DEFAULT_LOG_LEVEL)
                    .num_args(1),
            )
            .arg(
                Arg::new("verbose")
                    .long("verbose")
                    .short('V')
                    .help("Enable verbose output (sets log-level to debug)")
                    .action(ArgAction::SetTrue),
            )
    }

    #[test]
    fn test_constants() {
        assert_eq!(DEFAULT_LOG_LEVEL, "info");
        assert_eq!(LOG_LEVEL_VALUES.len(), 6);
        assert!(LOG_LEVEL_VALUES.contains(&"trace"));
        assert!(LOG_LEVEL_VALUES.contains(&"debug"));
        assert!(LOG_LEVEL_VALUES.contains(&"info"));
        assert!(LOG_LEVEL_VALUES.contains(&"warn"));
        assert!(LOG_LEVEL_VALUES.contains(&"error"));
        assert!(LOG_LEVEL_VALUES.contains(&"off"));
    }

    #[test]
    fn test_extract_log_level_default() {
        let cmd = create_test_command();
        let matches = cmd.try_get_matches_from(["test"]).unwrap();

        let log_level = extract_log_level(&matches);
        assert_eq!(log_level, "info");
    }

    #[test]
    fn test_extract_log_level_verbose() {
        let cmd = create_test_command();
        let matches = cmd.try_get_matches_from(["test", "--verbose"]).unwrap();

        let log_level = extract_log_level(&matches);
        assert_eq!(log_level, "debug");
    }

    #[test]
    fn test_extract_log_level_explicit() {
        let cmd = create_test_command();
        let matches = cmd
            .try_get_matches_from(["test", "--log-level", "warn"])
            .unwrap();

        let log_level = extract_log_level(&matches);
        assert_eq!(log_level, "warn");
    }

    #[test]
    fn test_verbose_overrides_log_level() {
        let cmd = create_test_command();
        let matches = cmd
            .try_get_matches_from(["test", "--log-level", "error", "--verbose"])
            .unwrap();

        let log_level = extract_log_level(&matches);
        assert_eq!(log_level, "debug");
    }
}
