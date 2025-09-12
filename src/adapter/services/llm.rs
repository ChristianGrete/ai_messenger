use crate::adapter::runtime::WasmRuntime;
use crate::adapter::traits::{AdapterService, LlmAdapter, ModelInfo, ServiceError};
use crate::config::schema::ServiceAdapterConfig;
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

/// LLM adapter wrapper providing typed interface to WASM instances
pub struct LlmAdapterWrapper {
    runtime: Arc<RwLock<WasmRuntime>>,
    provider: String,
    version: String,
    service_name: String,
}

impl LlmAdapterWrapper {
    /// Create new LLM adapter wrapper
    pub async fn new(
        runtime: &Arc<RwLock<WasmRuntime>>,
        config: &ServiceAdapterConfig,
        data_dir: &Path,
        service_name: &str,
    ) -> Result<Self, ServiceError> {
        let module_path = config.module_path(data_dir, service_name);
        let config_json = config
            .config_as_json()
            .map_err(|e| ServiceError::InvalidConfig(e.to_string()))?;

        // Load the WASM module with proper WIT bindings
        {
            let mut runtime_guard = runtime.write().await;
            runtime_guard
                .load_adapter(service_name, &module_path, &config_json)
                .await?;
        }
        Ok(LlmAdapterWrapper {
            runtime: runtime.clone(),
            provider: config.provider.clone(),
            version: config.version.clone(),
            service_name: service_name.to_string(),
        })
    }
}

#[async_trait]
impl AdapterService for LlmAdapterWrapper {
    fn service_name(&self) -> &'static str {
        "llm"
    }

    fn provider_name(&self) -> &str {
        &self.provider
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn is_ready(&self) -> bool {
        // TODO: Check actual WASM instance readiness
        true
    }

    async fn shutdown(&mut self) -> Result<(), ServiceError> {
        // The runtime handles instance cleanup
        Ok(())
    }
}

#[async_trait]
impl LlmAdapter for LlmAdapterWrapper {
    async fn send_message(&mut self, message: &str) -> Result<String, ServiceError> {
        // Get WASM instance for this provider
        let mut runtime_guard = self.runtime.write().await;
        let instance = runtime_guard
            .get_instance_mut(&self.service_name, &self.provider)
            .ok_or_else(|| {
                ServiceError::ServiceUnavailable(format!(
                    "WASM adapter not loaded for provider: {}",
                    self.provider
                ))
            })?;

        // Prepare chat request for WASM adapter
        let chat_request = serde_json::json!({
            "messages": [
                {
                    "role": "user",
                    "content": message
                }
            ],
            "model": "qwen2.5:7b-instruct"
        });

        // Call WASM function to prepare HTTP request
        let http_config = instance
            .call_llm_function("prepare_request", chat_request)
            .await?;

        // Extract HTTP configuration from WASM response
        let url = http_config["url"].as_str().ok_or_else(|| {
            ServiceError::ExecutionError("Missing URL in WASM response".to_string())
        })?;

        let headers = http_config["headers"].as_object().ok_or_else(|| {
            ServiceError::ExecutionError("Missing headers in WASM response".to_string())
        })?;

        let body = http_config["body"].as_str().ok_or_else(|| {
            ServiceError::ExecutionError("Missing body in WASM response".to_string())
        })?;

        // Release the lock before making HTTP request
        drop(runtime_guard);

        // Make HTTP request using WASM-prepared configuration
        let client = reqwest::Client::new();
        let mut request_builder = client.post(url);

        // Add headers from WASM adapter
        for (key, value) in headers {
            if let Some(value_str) = value.as_str() {
                request_builder = request_builder.header(key, value_str);
            }
        }

        let response = request_builder
            .body(body.to_string())
            .send()
            .await
            .map_err(|e| ServiceError::ServiceUnavailable(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ServiceError::ServiceUnavailable(format!(
                "API error: {}",
                response.status()
            )));
        }

        let response_body = response.text().await.map_err(|e| {
            ServiceError::ServiceUnavailable(format!("Failed to read response: {}", e))
        })?;

        // Call WASM function to parse response
        let mut runtime_guard = self.runtime.write().await;
        let instance = runtime_guard
            .get_instance_mut(&self.service_name, &self.provider)
            .ok_or_else(|| {
                ServiceError::ServiceUnavailable(format!(
                    "WASM adapter not loaded for provider: {}",
                    self.provider
                ))
            })?;

        let response_json = serde_json::json!(response_body);
        let parsed_response = instance
            .call_llm_function("parse_response", response_json)
            .await?;

        // Extract message content from parsed response
        let content = parsed_response["message"]["content"]
            .as_str()
            .ok_or_else(|| {
                ServiceError::ExecutionError(
                    "Failed to extract content from WASM response".to_string(),
                )
            })?;

        Ok(content.to_string())
    }

    async fn get_model_info(&self) -> Result<ModelInfo, ServiceError> {
        let runtime = self.runtime.read().await;

        if let Some(instance) = runtime.get_instance(&self.service_name, &self.provider) {
            if !instance.is_ready() {
                return Err(ServiceError::ServiceUnavailable(
                    "LLM adapter not ready".to_string(),
                ));
            }

            // TODO: Call actual WASM function to get model info
            // For now, return placeholder info
            Ok(ModelInfo {
                name: format!("{}_model", self.provider),
                version: self.version.clone(),
                context_length: Some(4096),
                parameters: Some("7B".to_string()),
            })
        } else {
            Err(ServiceError::ServiceUnavailable(
                "LLM adapter instance not found".to_string(),
            ))
        }
    }
}
