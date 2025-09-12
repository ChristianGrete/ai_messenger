use async_trait::async_trait;
use std::fmt;
use thiserror::Error;

/// Common error type for all adapter operations
#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Adapter initialization failed: {0}")]
    InitializationFailed(String),
    #[error("WASM module execution error: {0}")]
    ExecutionError(String),
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

/// Base trait for all service adapters
#[allow(async_fn_in_trait)] // Used only within our crate, Send bounds not critical
pub trait AdapterService: Send + Sync {
    /// Service name (e.g., "llm", "storage", "tts")
    fn service_name(&self) -> &'static str;

    /// Provider name (e.g., "ollama", "openai", "s3")
    fn provider_name(&self) -> &str;

    /// Adapter version
    fn version(&self) -> &str;

    /// Check if the adapter is ready to handle requests
    fn is_ready(&self) -> bool;

    /// Graceful shutdown of the adapter
    async fn shutdown(&mut self) -> Result<(), ServiceError>;
}
/// Trait for LLM service adapters
#[async_trait]
pub trait LlmAdapter: AdapterService {
    /// Send a message and get response
    async fn send_message(&mut self, message: &str) -> Result<String, ServiceError>;

    /// Get model information
    async fn get_model_info(&self) -> Result<ModelInfo, ServiceError>;

    // Stream message response (future enhancement)
    // async fn stream_message(&mut self, message: &str) -> Result<impl Stream<Item = String>, ServiceError>;
}
/// Trait for storage service adapters
#[async_trait]
pub trait StorageAdapter: AdapterService {
    /// Store data with a key
    async fn store(&mut self, key: &str, data: &[u8]) -> Result<(), ServiceError>;

    /// Retrieve data by key
    async fn retrieve(&self, key: &str) -> Result<Vec<u8>, ServiceError>;

    /// Delete data by key
    async fn delete(&mut self, key: &str) -> Result<(), ServiceError>;

    /// Check if key exists
    async fn exists(&self, key: &str) -> Result<bool, ServiceError>;

    /// List all keys with optional prefix
    async fn list_keys(&self, prefix: Option<&str>) -> Result<Vec<String>, ServiceError>;
}

/// Model information returned by LLM adapters
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub version: String,
    pub context_length: Option<u32>,
    pub parameters: Option<String>,
}

impl fmt::Display for ModelInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.version)
    }
}
