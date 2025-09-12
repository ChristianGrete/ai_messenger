//! # ai_messenger
//!
//! A pluggable AI messaging platform with WASM-based adapter architecture.
//!
//! ## Features
//!
//! - **Open Core Architecture**: Free privacy-respecting foundation with optional premium features
//! - **WASM Isolation**: All adapters run in sandboxed WASM modules for security
//! - **Composable Design**: Mix and match AI providers, storage backends, and encryption methods
//! - **Plugin Ecosystem**: Community-driven adapter marketplace
//!
//! ## Quick Start
//!
//! ### As a Library (embedded in your app):
//! ```rust,no_run
//! use ai_messenger::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Your app controls logging
//!     tracing_subscriber::fmt::init();
//!
//!     // Initialize ai_messenger (without touching global logging)
//!     ai_messenger::init()?;
//!
//!     let config = Config::default();
//!     // server::start_with_config(config).await?;
//!     Ok(())
//! }
//! ```
//!
//! ### As a Standalone App:
//! ```rust,no_run
//! use ai_messenger::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // ai_messenger controls logging
//!     ai_messenger::init_with_logging("info")?;
//!
//!     let config = Config::default();
//!     // server::start_with_config(config).await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture
//!
//! The platform follows a layered architecture:
//!
//! ```text
//! AI Services ← WASM AI Adapters
//!      ↓
//! Core Logic (Rust)
//!      ↓
//! WASM Crypto Adapters → WASM Storage Adapters → Physical Storage
//! ```

// Re-export shared modules that are part of the public API
pub mod adapter;
pub mod config;
pub mod utils;
// TODO: Uncomment when implemented
// pub mod server;

// Library-specific modules (organized in src/library/)
mod library;

// Clean re-exports at crate root for excellent DX
pub use library::{
    error::{Error, Result},
    init::{init, init_with_logging},
    prelude,
    types::*,
};

// Re-export high-level API functions at crate root
// TODO: Uncomment when implemented
// pub use library::api::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lib_init() {
        // Test that library initialization works
        assert!(init().is_ok());
    }

    #[test]
    fn test_lib_init_with_logging() {
        // Test that standalone initialization works
        assert!(init_with_logging("debug").is_ok());
        // Should be safe to call multiple times
        assert!(init_with_logging("info").is_ok());
    }

    #[test]
    fn test_public_api_exists() {
        // Ensure our main modules are accessible
        let _config = config::schema::Config::default();
        // TODO: Add more as we implement
    }

    #[test]
    fn test_prelude_imports() {
        // Test that prelude brings in the expected items
        use crate::prelude::*;

        let _config = Config::default();
        assert!(init().is_ok());

        // Test that tracing macros are available
        info!("Test log message");
        debug!("Test debug message");
    }
}
