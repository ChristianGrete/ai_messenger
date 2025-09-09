use std::env;
use std::path::{Path, PathBuf};

/// Expand path with support for home directory and config-relative paths
///
/// Supports:
/// - `~` at the beginning of the path (replaced with home directory)
/// - `$HOME` anywhere in the path (replaced with home directory)
/// - `./` and `../` at the beginning (relative to config directory)
/// - Other relative paths without leading slash (relative to config directory)
///
/// If config_dir is None, only home expansion is performed.
pub fn expand_path<P: AsRef<Path>>(path: P, config_dir: Option<&Path>) -> PathBuf {
    let path = path.as_ref();
    let path_str = path.to_string_lossy();

    // First priority: Home directory expansion
    if path_str.starts_with("~") || path_str.contains("$HOME") {
        return expand_home(path);
    }

    // Second priority: Config-relative paths (if config_dir is available)
    if let Some(config_dir) = config_dir {
        // Check if it's a relative path (not absolute)
        if !path.is_absolute() {
            // Relative paths: ./foo, ../foo, foo/bar (but not ~/foo which was handled above)
            let joined = config_dir.join(path);

            // Try to canonicalize (works only if path exists), otherwise clean manually
            if let Ok(canonical) = joined.canonicalize() {
                return canonical;
            } else {
                // If canonicalize fails (path doesn't exist), clean the path manually
                // This resolves .. and . components without requiring the path to exist
                let mut components = Vec::new();
                for component in joined.components() {
                    match component {
                        std::path::Component::ParentDir => {
                            components.pop();
                        }
                        std::path::Component::CurDir => {
                            // Skip current directory components
                        }
                        _ => {
                            components.push(component);
                        }
                    }
                }
                return components.iter().collect();
            }
        }
    }

    // Default: absolute paths or fallback when no config_dir
    path.to_path_buf()
}

/// Expand home directory placeholders in a path
///
/// Supports:
/// - `~` at the beginning of the path (replaced with home directory)
/// - `$HOME` anywhere in the path (replaced with home directory)
///
/// If the home directory cannot be determined, the path is returned unchanged.
pub fn expand_home<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();
    let path_str = path.to_string_lossy();

    // Get home directory
    let home_dir = match env::var("HOME") {
        Ok(home) => home,
        Err(_) => {
            // Fallback to dirs crate for cross-platform support
            match dirs::home_dir() {
                Some(home) => home.to_string_lossy().to_string(),
                None => return path.to_path_buf(), // No expansion possible
            }
        }
    };

    let expanded = if path_str.starts_with("~") {
        // Replace ~ at the beginning with home directory
        path_str.replacen("~", &home_dir, 1)
    } else if path_str.contains("$HOME") {
        // Replace $HOME anywhere in the path, but only whole tokens
        // We need to be careful not to replace partial matches like $HOMECOMING
        replace_home_variable(&path_str, &home_dir)
    } else {
        // No expansion needed
        return path.to_path_buf();
    };

    PathBuf::from(expanded)
}

/// Replace $HOME variable in a path string, but only as a whole token
fn replace_home_variable(path: &str, home_dir: &str) -> String {
    let mut result = String::new();
    let mut chars = path.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '$' {
            // Check if this is followed by "HOME"
            let mut temp_chars = chars.clone();
            let next_chars: String = temp_chars.by_ref().take(4).collect();

            if next_chars == "HOME" {
                // Check what comes after "HOME"
                let char_after_home = temp_chars.peek();

                // Only consider it NOT a match if it's followed by alphanumeric characters
                // This way $HOME in filenames like backup_$HOME_file.txt still gets replaced
                // but $HOMECOMING doesn't
                if char_after_home.is_some() && char_after_home.unwrap().is_alphanumeric() {
                    // This looks like $HOME followed by more letters (like $HOMECOMING), don't replace
                    result.push(ch);
                } else {
                    // This is a valid $HOME token (followed by separator, underscore, or end), replace it
                    result.push_str(home_dir);
                    // Skip the "HOME" characters
                    for _ in 0..4 {
                        chars.next();
                    }
                }
            } else {
                // Not followed by "HOME", just add the $
                result.push(ch);
            }
        } else {
            result.push(ch);
        }
    }

    result
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_expand_home_tilde() {
        let path = "~/test/path";
        let expanded = expand_home(path);

        // Should expand ~ to home directory
        assert!(!expanded.to_string_lossy().starts_with("~"));
        assert!(expanded.to_string_lossy().ends_with("/test/path"));
    }

    #[test]
    fn test_expand_home_dollar_home() {
        let path = "$HOME/test/path";
        let expanded = expand_home(path);

        // Should expand $HOME to home directory
        assert!(!expanded.to_string_lossy().contains("$HOME"));
        assert!(expanded.to_string_lossy().ends_with("/test/path"));
    }

    #[test]
    fn test_expand_home_dollar_home_middle() {
        let path = "/prefix/$HOME/suffix";
        let expanded = expand_home(path);

        // Should expand $HOME in the middle
        assert!(!expanded.to_string_lossy().contains("$HOME"));
        assert!(expanded.to_string_lossy().starts_with("/prefix/"));
        assert!(expanded.to_string_lossy().ends_with("/suffix"));
    }

    #[test]
    fn test_expand_home_no_expansion() {
        let path = "/absolute/path/without/home";
        let expanded = expand_home(path);

        // Should remain unchanged
        assert_eq!(expanded, PathBuf::from(path));
    }

    #[test]
    fn test_expand_home_relative_path() {
        let path = "relative/path";
        let expanded = expand_home(path);

        // Should remain unchanged
        assert_eq!(expanded, PathBuf::from(path));
    }

    #[test]
    fn test_expand_home_tilde_not_at_start() {
        let path = "/some/~/path";
        let expanded = expand_home(path);

        // Should not expand ~ in the middle
        assert_eq!(expanded, PathBuf::from(path));
    }

    #[test]
    fn test_expand_home_pathbuf_input() {
        let path = PathBuf::from("~/test");
        let expanded = expand_home(&path);

        // Should work with PathBuf input
        assert!(!expanded.to_string_lossy().starts_with("~"));
        assert!(expanded.to_string_lossy().ends_with("/test"));
    }

    #[test]
    fn test_expand_home_with_actual_home() {
        // Test with actual HOME environment variable if available
        if let Ok(home) = env::var("HOME") {
            let path = "~/documents";
            let expanded = expand_home(path);

            assert_eq!(expanded, PathBuf::from(format!("{}/documents", home)));
        }
    }

    #[test]
    fn test_expand_home_multiple_dollar_home() {
        let path = "$HOME/test/$HOME/path";
        let expanded = expand_home(path);

        // Should replace all occurrences of $HOME
        assert!(!expanded.to_string_lossy().contains("$HOME"));
    }

    #[test]
    fn test_expand_home_empty_path() {
        let path = "";
        let expanded = expand_home(path);

        // Should handle empty path gracefully
        assert_eq!(expanded, PathBuf::from(""));
    }

    #[test]
    fn test_expand_home_just_tilde() {
        let path = "~";
        let expanded = expand_home(path);

        // Should expand just ~ to home directory
        if let Ok(home) = env::var("HOME") {
            assert_eq!(expanded, PathBuf::from(home));
        } else if let Some(home) = dirs::home_dir() {
            assert_eq!(expanded, home);
        }
    }

    #[test]
    fn test_expand_home_just_dollar_home() {
        let path = "$HOME";
        let expanded = expand_home(path);

        // Should expand just $HOME to home directory
        if let Ok(home) = env::var("HOME") {
            assert_eq!(expanded, PathBuf::from(home));
        } else if let Some(home) = dirs::home_dir() {
            assert_eq!(expanded, home);
        }
    }

    #[test]
    fn test_expand_home_mixed_separators() {
        // Test with different path separators (important for cross-platform)
        let path = "~/Documents/My Projects/app";
        let expanded = expand_home(path);

        assert!(!expanded.to_string_lossy().starts_with("~"));
        assert!(expanded.to_string_lossy().contains("Documents"));
        assert!(expanded.to_string_lossy().contains("My Projects"));
    }

    #[test]
    fn test_expand_home_nested_dollar_home_in_filename() {
        // Test when $HOME appears in a filename itself
        let path = "/tmp/backup_$HOME_file.txt";
        let expanded = expand_home(path);

        assert!(!expanded.to_string_lossy().contains("$HOME"));
        assert!(expanded.to_string_lossy().starts_with("/tmp/backup_"));
        assert!(expanded.to_string_lossy().ends_with("_file.txt"));
    }

    #[test]
    fn test_expand_home_case_sensitivity() {
        // $HOME should be case-sensitive
        let path = "/prefix/$home/suffix";
        let expanded = expand_home(path);

        // Should NOT expand $home (lowercase)
        assert_eq!(expanded, PathBuf::from(path));
        assert!(expanded.to_string_lossy().contains("$home"));
    }

    #[test]
    fn test_expand_home_partial_match() {
        // Test that we don't expand partial matches like $HOMECOMING
        let path = "/path/to/$HOMECOMING/dir";
        let expanded = expand_home(path);

        // Should NOT expand $HOMECOMING
        assert_eq!(expanded, PathBuf::from(path));
        assert!(expanded.to_string_lossy().contains("$HOMECOMING"));
    }

    #[test]
    fn test_expand_home_unicode_paths() {
        // Test with unicode characters
        let path = "~/Documents/测试文件夹/äöü";
        let expanded = expand_home(path);

        assert!(!expanded.to_string_lossy().starts_with("~"));
        assert!(expanded.to_string_lossy().contains("测试文件夹"));
        assert!(expanded.to_string_lossy().contains("äöü"));
    }

    #[test]
    fn test_expand_home_windows_style_path() {
        // Test with backslashes (should work on Unix too)
        let path = "~\\Documents\\app";
        let expanded = expand_home(path);

        assert!(!expanded.to_string_lossy().starts_with("~"));
        // Should preserve the backslashes as literal characters on Unix
        assert!(expanded.to_string_lossy().contains("Documents"));
    }

    #[test]
    fn test_expand_home_env_var_without_home() {
        // Test behavior when HOME is unset
        let original_home = env::var("HOME").ok();

        // Temporarily unset HOME (unsafe operation)
        unsafe {
            env::remove_var("HOME");
        }

        let path = "~/test";
        let expanded = expand_home(path);

        // Should either expand via dirs::home_dir() or remain unchanged
        // We can't predict exact behavior but it shouldn't panic
        assert!(!expanded.as_os_str().is_empty());

        // Restore original HOME if it existed (unsafe operation)
        if let Some(home) = original_home {
            unsafe {
                env::set_var("HOME", home);
            }
        }
    }
    #[test]
    fn test_expand_home_complex_nesting() {
        // Complex case with multiple nested variables
        let path = "$HOME/configs/$HOME/app";
        let expanded = expand_home(path);

        // Should replace both instances of $HOME
        assert!(!expanded.to_string_lossy().contains("$HOME"));
        assert!(expanded.to_string_lossy().contains("/configs/"));

        // Path should appear twice in the expanded version
        if let Ok(home) = env::var("HOME") {
            let path_str = expanded.to_string_lossy();
            let home_count = path_str.matches(&home).count();
            assert_eq!(home_count, 2);
        }
    }

    #[test]
    fn test_expand_path_config_relative() {
        let config_dir = std::path::PathBuf::from("/config/dir");

        // Test relative paths with ./
        let result = expand_path("./data", Some(&config_dir));
        assert_eq!(result, std::path::PathBuf::from("/config/dir/data"));

        // Test relative paths with ../
        let result = expand_path("../other", Some(&config_dir));
        assert_eq!(result, std::path::PathBuf::from("/config/other"));

        // Test simple relative path
        let result = expand_path("subdir/file", Some(&config_dir));
        assert_eq!(result, std::path::PathBuf::from("/config/dir/subdir/file"));
    }

    #[test]
    fn test_expand_path_home_takes_priority() {
        let config_dir = std::path::PathBuf::from("/config/dir");

        // Home expansion should take priority over config-relative
        let result = expand_path("~/documents", Some(&config_dir));
        if let Some(home) = dirs::home_dir() {
            let expected = home.join("documents");
            assert_eq!(result, expected);
        }

        // $HOME expansion should also take priority
        let result = expand_path("$HOME/documents", Some(&config_dir));
        if let Ok(home) = env::var("HOME") {
            let expected = std::path::PathBuf::from(home).join("documents");
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_expand_path_absolute_unchanged() {
        let config_dir = std::path::PathBuf::from("/config/dir");

        // Absolute paths should remain unchanged
        let result = expand_path("/absolute/path", Some(&config_dir));
        assert_eq!(result, std::path::PathBuf::from("/absolute/path"));
    }

    #[test]
    fn test_expand_path_no_config_dir() {
        // Without config_dir, only home expansion should work
        let result = expand_path("./relative", None);
        assert_eq!(result, std::path::PathBuf::from("./relative"));

        let result = expand_path("~", None);
        if let Some(home) = dirs::home_dir() {
            assert_eq!(result, home);
        }
    }
}
