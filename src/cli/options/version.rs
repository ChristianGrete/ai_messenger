use clap::Command;

pub fn apply(cmd: Command) -> Command {
    cmd.long_version(build_version_string())
}

fn build_version_string() -> &'static str {
    use std::sync::OnceLock;

    static VERSION: OnceLock<String> = OnceLock::new();

    VERSION.get_or_init(|| {
        format!(
            "{}\ncommit: {}\nbuilt: {}",
            env!("CARGO_PKG_VERSION"),
            option_env!("GIT_SHA").unwrap_or("unknown"),
            option_env!("BUILD_TIME").unwrap_or("unknown"),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Command;

    #[test]
    fn test_apply_adds_long_version() {
        let cmd = Command::new("test");
        let modified_cmd = apply(cmd);

        // Should have long version set
        assert!(modified_cmd.get_long_version().is_some());
    }

    #[test]
    fn test_build_version_string_contains_cargo_version() {
        let version_string = build_version_string();

        // Should contain the cargo package version
        assert!(version_string.contains(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn test_build_version_string_format() {
        let version_string = build_version_string();

        // Should have expected format with newlines
        assert!(version_string.contains("\ncommit: "));
        assert!(version_string.contains("\nbuilt: "));
    }

    #[test]
    fn test_build_version_string_handles_missing_env_vars() {
        let version_string = build_version_string();

        // Should contain "unknown" for missing env vars (since they're not set in tests)
        assert!(version_string.contains("unknown"));
    }

    #[test]
    fn test_build_version_string_is_static() {
        let version1 = build_version_string();
        let version2 = build_version_string();

        // Should return the same reference (static/cached)
        assert_eq!(version1.as_ptr(), version2.as_ptr());
    }

    #[test]
    fn test_version_string_multiline() {
        let version_string = build_version_string();
        let lines: Vec<&str> = version_string.lines().collect();

        // Should have exactly 3 lines
        assert_eq!(lines.len(), 3);
        assert!(!lines[0].is_empty()); // Version line
        assert!(lines[1].starts_with("commit: "));
        assert!(lines[2].starts_with("built: "));
    }
}
