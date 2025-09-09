use clap::{Arg, ArgAction, Command};

pub fn build() -> Command {
    let cmd = Command::new("ai_messenger")
        .about("Multi-provider AI messaging service")
        .disable_version_flag(true)
        .disable_help_flag(true)
        .disable_help_subcommand(true)
        .arg(
            Arg::new("help")
                .long("help")
                .short('h')
                .help("Print help")
                .action(ArgAction::Help),
        )
        .arg(
            Arg::new("version")
                .long("version")
                .short('v')
                .help("Print version")
                .action(ArgAction::Version),
        )
        .subcommand(super::commands::cache::command())
        .subcommand(super::commands::data::command())
        .subcommand(
            Command::new("help")
                .about("Print this message or the help of the given command")
                .arg(
                    Arg::new("command")
                        .help("Command to show help for")
                        .value_name("COMMAND")
                        .num_args(0..=1),
                ),
        )
        .subcommand(super::commands::serve::command());

    let cmd = super::options::help::apply(cmd);
    super::options::version::apply(cmd)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::error::ErrorKind;

    #[test]
    fn test_build_creates_main_command() {
        let cmd = build();

        assert_eq!(cmd.get_name(), "ai_messenger");
        let about_str = format!("{}", cmd.get_about().unwrap());
        assert_eq!(about_str, "Multi-provider AI messaging service");
    }

    #[test]
    fn test_main_command_has_required_args() {
        let cmd = build();

        // Should have help and version arguments
        assert!(cmd.get_arguments().any(|arg| arg.get_id() == "help"));
        assert!(cmd.get_arguments().any(|arg| arg.get_id() == "version"));
    }

    #[test]
    fn test_main_command_has_all_subcommands() {
        let cmd = build();

        let subcommand_names: Vec<&str> = cmd.get_subcommands().map(|sub| sub.get_name()).collect();

        // Should have all expected subcommands in alphabetical order
        assert!(subcommand_names.contains(&"cache"));
        assert!(subcommand_names.contains(&"data"));
        assert!(subcommand_names.contains(&"serve"));
        assert!(subcommand_names.contains(&"help"));
        assert_eq!(subcommand_names.len(), 4);
    }

    #[test]
    fn test_subcommands_alphabetical_order() {
        let cmd = build();

        let subcommand_names: Vec<&str> = cmd.get_subcommands().map(|sub| sub.get_name()).collect();

        // Should be in alphabetical order: cache, data, help, serve
        assert_eq!(subcommand_names, vec!["cache", "data", "help", "serve"]);
    }

    #[test]
    fn test_help_argument_properties() {
        let cmd = build();

        let help_arg = cmd
            .get_arguments()
            .find(|arg| arg.get_id() == "help")
            .unwrap();

        assert!(matches!(help_arg.get_action(), ArgAction::Help));
        assert_eq!(help_arg.get_short(), Some('h'));
        assert_eq!(help_arg.get_long(), Some("help"));
    }

    #[test]
    fn test_version_argument_properties() {
        let cmd = build();

        let version_arg = cmd
            .get_arguments()
            .find(|arg| arg.get_id() == "version")
            .unwrap();

        assert!(matches!(version_arg.get_action(), ArgAction::Version));
        assert_eq!(version_arg.get_short(), Some('v'));
        assert_eq!(version_arg.get_long(), Some("version"));
    }

    #[test]
    fn test_help_subcommand_properties() {
        let cmd = build();

        let help_subcommand = cmd
            .get_subcommands()
            .find(|sub| sub.get_name() == "help")
            .unwrap();

        let about_str = format!("{}", help_subcommand.get_about().unwrap());
        assert_eq!(
            about_str,
            "Print this message or the help of the given command"
        );

        // Help subcommand should have a command argument
        assert!(
            help_subcommand
                .get_arguments()
                .any(|arg| arg.get_id() == "command")
        );
    }

    #[test]
    fn test_disabled_flags() {
        let cmd = build();

        // Auto flags should be disabled
        assert!(cmd.is_disable_help_flag_set());
        assert!(cmd.is_disable_version_flag_set());
        assert!(cmd.is_disable_help_subcommand_set());
    }

    #[test]
    fn test_help_flag_works() {
        let cmd = build();
        let result = cmd.try_get_matches_from(["ai_messenger", "--help"]);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::DisplayHelp);
    }

    #[test]
    fn test_short_help_flag_works() {
        let cmd = build();
        let result = cmd.try_get_matches_from(["ai_messenger", "-h"]);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::DisplayHelp);
    }

    #[test]
    fn test_version_flag_works() {
        let cmd = build();
        let result = cmd.try_get_matches_from(["ai_messenger", "--version"]);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::DisplayVersion);
    }

    #[test]
    fn test_short_version_flag_works() {
        let cmd = build();
        let result = cmd.try_get_matches_from(["ai_messenger", "-v"]);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::DisplayVersion);
    }

    #[test]
    fn test_subcommand_parsing() {
        // Test each subcommand can be parsed individually
        for subcommand_name in ["serve", "cache", "data", "help"] {
            let cmd = build();
            let matches = cmd
                .try_get_matches_from(["ai_messenger", subcommand_name])
                .unwrap();
            assert!(matches.subcommand_name() == Some(subcommand_name));
        }
    }

    #[test]
    fn test_help_subcommand_with_argument() {
        let cmd = build();
        let matches = cmd
            .try_get_matches_from(["ai_messenger", "help", "serve"])
            .unwrap();

        if let Some(("help", sub_matches)) = matches.subcommand() {
            assert_eq!(
                sub_matches.get_one::<String>("command"),
                Some(&"serve".to_string())
            );
        } else {
            panic!("Expected help subcommand");
        }
    }

    #[test]
    fn test_help_subcommand_without_argument() {
        let cmd = build();
        let matches = cmd.try_get_matches_from(["ai_messenger", "help"]).unwrap();

        if let Some(("help", sub_matches)) = matches.subcommand() {
            assert!(sub_matches.get_one::<String>("command").is_none());
        } else {
            panic!("Expected help subcommand");
        }
    }

    #[test]
    fn test_no_subcommand() {
        let cmd = build();
        let matches = cmd.try_get_matches_from(["ai_messenger"]).unwrap();

        // Should parse successfully with no subcommand
        assert!(matches.subcommand().is_none());
    }

    #[test]
    fn test_options_are_applied() {
        let cmd = build();

        // Should have long version (from version::apply)
        assert!(cmd.get_long_version().is_some());
    }

    #[test]
    fn test_argument_count() {
        let cmd = build();

        // Main command should have exactly 2 arguments: help and version
        assert_eq!(cmd.get_arguments().count(), 2);
    }

    #[test]
    fn test_subcommand_count() {
        let cmd = build();

        // Should have exactly 4 subcommands
        assert_eq!(cmd.get_subcommands().count(), 4);
    }

    #[test]
    fn test_help_subcommand_command_argument() {
        let cmd = build();
        let help_subcommand = cmd
            .get_subcommands()
            .find(|sub| sub.get_name() == "help")
            .unwrap();

        let command_arg = help_subcommand
            .get_arguments()
            .find(|arg| arg.get_id() == "command")
            .unwrap();

        // Command argument should be optional and take 0 or 1 values
        assert!(!command_arg.is_required_set());
        assert_eq!(command_arg.get_num_args(), Some((0..=1).into()));
    }
}
