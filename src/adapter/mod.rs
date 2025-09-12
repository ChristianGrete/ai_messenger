// WASM Adapter System - Public API exports
//
// This module provides the public interface for the WASM adapter system,
// enabling config-driven loading and management of service adapters.

pub mod runtime;
pub mod services;
pub mod traits;

#[cfg(test)]
mod tests;

// Re-export key types for public API
pub use runtime::WasmRuntime;
pub use services::AdapterRegistry;
pub use traits::{AdapterService, ServiceError};
