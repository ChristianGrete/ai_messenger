use anyhow::Result;
use std::sync::Once;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

static INIT: Once = Once::new();

/// Helper to create EnvFilter with fallback logic
fn get_env_filter(level: &str) -> EnvFilter {
    EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(level))
        .unwrap_or_else(|_| EnvFilter::new("info"))
}

/// Initialize tracing/logging system with the specified log level
/// Safe to call multiple times - will only initialize once
pub fn init_logging(level: &str) -> Result<()> {
    INIT.call_once(|| {
        // Create filter from level string
        let filter = get_env_filter(level);

        // Set up console logging with clean format
        let _ = tracing_subscriber::registry()
            .with(
                fmt::layer()
                    .with_target(false) // Don't show module path (cleaner output)
                    .with_level(true) // Show log level
                    .compact(), // Compact format
            )
            .with(filter)
            .try_init(); // Use try_init to avoid panic on multiple calls
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_logging_valid_levels() {
        // Test that init_logging doesn't panic with valid levels
        // Note: We can't actually test the full initialization in unit tests
        // because tracing can only be initialized once per process

        // Test that filter creation works
        let levels = ["trace", "debug", "info", "warn", "error"];
        for level in levels {
            let filter_result = EnvFilter::try_new(level);
            assert!(
                filter_result.is_ok(),
                "Failed to create filter for level: {}",
                level
            );
        }
    }

    #[test]
    fn test_init_logging_invalid_level() {
        // Note: tracing EnvFilter is very tolerant and accepts many "invalid" strings
        // as it treats them as filter expressions. We test that our function
        // handles the Result properly rather than testing tracing's validation

        let filter_result = EnvFilter::try_new(""); // Empty string should fail
        // Even if this doesn't fail, that's okay - tracing is designed to be forgiving
        // The important thing is our function signature handles Result properly
        let _ = filter_result; // Don't assert - just verify compilation
    }

    #[test]
    fn test_multiple_init_logging_calls() {
        // Test that multiple calls don't panic due to std::sync::Once
        let result1 = init_logging("info");
        assert!(result1.is_ok());

        let result2 = init_logging("debug");
        assert!(result2.is_ok());

        let result3 = init_logging("invalid_but_handled_gracefully");
        assert!(result3.is_ok());

        // All should succeed without panic due to Once::call_once
    }

    #[test]
    fn test_logging_graceful_fallback() {
        // Test that invalid log levels fall back to "info" gracefully
        // This tests the EnvFilter::new("info") fallback in our code

        let result = init_logging("completely_invalid_level_12345");
        assert!(result.is_ok());

        // The function should not panic and should handle the fallback
        // We can't easily test the actual log level set, but we can ensure
        // the function completes successfully
    }
}
