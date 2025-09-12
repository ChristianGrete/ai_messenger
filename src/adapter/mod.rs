//! WASM Adapter Runtime
//!
//! This module provides the WASM-based adapter architecture for ai_messenger.
//! All service adapters (LLM, storage, TTS, etc.) run in sandboxed WASM modules.

// Allow unused code during development phase
#![allow(unused_imports, dead_code)]

pub mod manifest;
pub mod runtime;
pub mod services;
pub mod traits;
pub mod wit;

#[cfg(test)]
pub mod tests;

pub use manifest::AdapterManifest;
pub use runtime::WasmRuntime;
pub use services::AdapterRegistry;
pub use traits::{AdapterService, ServiceError};
