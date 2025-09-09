/// Error types for the ai_messenger library.
pub use anyhow::{Error, Result};

// TODO: Add custom error types as needed when we implement server layer
// #[derive(Debug, thiserror::Error)]
// pub enum AdapterError {
//     #[error("WASM module failed to load: {0}")]
//     WasmLoadError(String),
//     #[error("Configuration error: {0}")]
//     ConfigError(String),
//     #[error("Network error: {0}")]
//     NetworkError(String),
//     // ...
// }
