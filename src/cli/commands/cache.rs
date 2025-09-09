use anyhow::Result;
use clap::{ArgMatches, Command};

pub fn command() -> Command {
    super::shared::create_path_command("cache", "Show the cache directory path")
}

pub async fn run(matches: &ArgMatches) -> Result<()> {
    super::shared::run_path_command(matches, crate::config::cache_dir).await
}

#[cfg(test)]
mod tests {
    use super::super::shared::test_utils;
    use super::*;

    #[test]
    fn test_command_creation() {
        test_utils::test_path_command("cache", "Show the cache directory path");
    }

    #[test]
    fn test_command_config_arg() {
        let cmd = command();
        test_utils::test_path_command_args(&cmd);
    }

    #[test]
    fn test_command_has_required_args() {
        let cmd = command();

        // Should have all expected arguments
        assert!(cmd.get_arguments().any(|arg| arg.get_id() == "config"));
        assert!(cmd.get_arguments().any(|arg| arg.get_id() == "help"));
        assert!(cmd.get_arguments().any(|arg| arg.get_id() == "log-level"));
        assert!(cmd.get_arguments().any(|arg| arg.get_id() == "verbose"));
    }

    #[test]
    fn test_command_help_arg() {
        let cmd = command();
        test_utils::test_path_command_args(&cmd);
    }

    #[test]
    fn test_command_help_flag() {
        test_utils::test_path_command_help_flags("cache");
    }

    #[test]
    fn test_command_parsing_no_args() {
        test_utils::test_path_command_parsing("cache");
    }

    #[test]
    fn test_command_parsing_with_config() {
        test_utils::test_path_command_parsing("cache");
    }

    #[tokio::test]
    async fn test_run_function_with_no_config() {
        let cmd = command();
        let matches = cmd.try_get_matches_from(["cache"]).unwrap();

        // Should run successfully and print a cache directory path
        let result = run(&matches).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_function_with_nonexistent_config() {
        let cmd = command();
        let matches = cmd
            .try_get_matches_from(["cache", "--config", "/nonexistent/config.toml"])
            .unwrap();

        // Should fail when config file doesn't exist
        let result = run(&matches).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_command_short_help_flag() {
        test_utils::test_path_command_help_flags("cache");
    }

    #[test]
    fn test_command_structure() {
        let cmd = command();

        // Should be properly configured
        assert_eq!(cmd.get_name(), "cache");
        assert!(cmd.is_disable_help_flag_set());

        // Should have exactly 4 arguments: config, help, log-level, verbose
        assert_eq!(cmd.get_arguments().count(), 4);
    }
}
