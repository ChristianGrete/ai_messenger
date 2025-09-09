use anyhow::Result;
use clap::{ArgMatches, Command};

pub fn command() -> Command {
    super::shared::create_path_command("data", "Show the data directory path")
}

pub async fn run(matches: &ArgMatches) -> Result<()> {
    super::shared::run_path_command(matches, crate::config::data_dir).await
}

#[cfg(test)]
mod tests {
    use super::super::shared::test_utils;
    use super::*;

    #[test]
    fn test_command_creation() {
        test_utils::test_path_command("data", "Show the data directory path");
    }

    #[test]
    fn test_command_config_arg() {
        let cmd = command();
        test_utils::test_path_command_args(&cmd);
    }

    #[test]
    fn test_command_has_required_args() {
        let cmd = command();

        // Should have all expected arguments (same structure as cache)
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
        test_utils::test_path_command_help_flags("data");
    }

    #[test]
    fn test_command_parsing_no_args() {
        test_utils::test_path_command_parsing("data");
    }

    #[test]
    fn test_command_parsing_with_config() {
        test_utils::test_path_command_parsing("data");
    }

    #[test]
    fn test_command_short_help_flag() {
        test_utils::test_path_command_help_flags("data");
    }

    #[test]
    fn test_command_structure() {
        let cmd = command();

        // Should be properly configured
        assert_eq!(cmd.get_name(), "data");
        assert!(cmd.is_disable_help_flag_set());

        // Should have exactly 4 arguments: config, help, log-level, verbose
        assert_eq!(cmd.get_arguments().count(), 4);
    }

    #[test]
    fn test_data_vs_cache_command_similarity() {
        let data_cmd = command();
        let cache_cmd = crate::cli::commands::cache::command();

        // Both commands should have same argument structure (but different names/about)
        assert_eq!(
            data_cmd.get_arguments().count(),
            cache_cmd.get_arguments().count()
        );

        // Both should have config and help args
        for cmd in [&data_cmd, &cache_cmd] {
            assert!(cmd.get_arguments().any(|arg| arg.get_id() == "config"));
            assert!(cmd.get_arguments().any(|arg| arg.get_id() == "help"));
        }
    }

    #[tokio::test]
    async fn test_run_function_with_no_config() {
        let cmd = command();
        let matches = cmd.try_get_matches_from(["data"]).unwrap();

        // Should run successfully and print a data directory path
        let result = run(&matches).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_function_with_nonexistent_config() {
        let cmd = command();
        let matches = cmd
            .try_get_matches_from(["data", "--config", "/nonexistent/config.toml"])
            .unwrap();

        // Should fail when config file doesn't exist
        let result = run(&matches).await;
        assert!(result.is_err());
    }
}
