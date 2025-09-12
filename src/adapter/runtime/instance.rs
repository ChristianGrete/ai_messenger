use crate::adapter::traits::ServiceError;
use wasmtime::{Store, component::Component};

/// WASM instance wrapper providing lifecycle management
pub struct WasmInstance {
    store: Store<InstanceState>,
    #[allow(dead_code)] // Will be used once WIT bindings are properly integrated
    component: Component,
    provider_name: String,
    version: String,
    is_ready: bool,
}

/// State shared with WASM instances
#[derive(Default)]
pub struct InstanceState {
    pub config_json: String,
    pub is_initialized: bool,
}

impl WasmInstance {
    /// Create new WASM instance from component and config
    pub fn new(
        engine: &wasmtime::Engine,
        component: Component,
        provider_name: String,
        version: String,
        config_json: String,
    ) -> Result<Self, ServiceError> {
        let state = InstanceState {
            config_json,
            is_initialized: false,
        };

        let store = Store::new(engine, state);

        Ok(WasmInstance {
            store,
            component,
            provider_name,
            version,
            is_ready: false,
        })
    }

    /// Initialize the WASM instance with configuration
    pub async fn initialize(&mut self) -> Result<(), ServiceError> {
        // Set fuel limit for security
        self.store
            .set_fuel(1_000_000)
            .map_err(|e| ServiceError::InitializationFailed(format!("Fuel setting failed: {e}")))?;

        // TODO: Complete WIT binding integration once type system is resolved
        // For now, mark as ready to enable WASM pipeline with adapter logic
        self.store.data_mut().is_initialized = true;
        self.is_ready = true;

        tracing::info!(
            "WASM instance initialized for provider: {}",
            self.provider_name
        );
        Ok(())
    }

    /// Call LLM adapter functions via manual HTTP abstraction
    /// TODO: Replace with proper WIT bindings once type issues are resolved
    pub async fn call_llm_function(
        &mut self,
        function_name: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, ServiceError> {
        if !self.is_ready {
            return Err(ServiceError::ServiceUnavailable(
                "Instance not initialized".to_string(),
            ));
        }

        // Add fuel for execution
        self.store
            .set_fuel(100_000)
            .map_err(|e| ServiceError::ExecutionError(format!("Fuel setting failed: {e}")))?;

        // For now, use simplified logic until WIT type conversion is working
        match function_name {
            "prepare_request" => {
                // Extract the basic request data
                let model = args["model"].as_str().unwrap_or("qwen2.5:7b-instruct");
                let messages = &args["messages"];

                // Build Ollama-compatible request
                let ollama_request = serde_json::json!({
                    "model": model,
                    "messages": messages,
                    "stream": false
                });

                // Return HTTP config for Ollama
                Ok(serde_json::json!({
                    "url": "http://localhost:11434/api/chat",
                    "headers": {
                        "Content-Type": "application/json"
                    },
                    "body": ollama_request.to_string()
                }))
            }
            "parse_response" => {
                // Parse Ollama response format
                let response_text = args.as_str().unwrap_or("{}");
                let response_json: serde_json::Value = serde_json::from_str(response_text)
                    .map_err(|e| {
                        ServiceError::ExecutionError(format!("Failed to parse response: {e}"))
                    })?;

                // Extract content in standardized format
                let content = response_json
                    .get("message")
                    .and_then(|m| m.get("content"))
                    .and_then(|c| c.as_str())
                    .unwrap_or("No content");

                Ok(serde_json::json!({
                    "message": {
                        "content": content,
                        "role": "assistant"
                    }
                }))
            }
            _ => Err(ServiceError::ExecutionError(format!(
                "Unknown function: {function_name}"
            ))),
        }
    }

    /// Get provider name
    pub fn provider_name(&self) -> &str {
        &self.provider_name
    }

    /// Get version
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Check if instance is ready
    pub fn is_ready(&self) -> bool {
        self.is_ready
    }

    /// Graceful shutdown
    pub async fn shutdown(self) -> Result<(), ServiceError> {
        // WASM instances are automatically cleaned up when dropped
        // This method exists for future resource cleanup if needed
        Ok(())
    }

    /// Get remaining fuel (for monitoring)
    pub fn remaining_fuel(&self) -> Option<u64> {
        self.store.get_fuel().ok()
    }
}
