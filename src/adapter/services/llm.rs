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

        // Load the WASM module
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
        let runtime = self.runtime.read().await;

        if let Some(instance) = runtime.get_instance(&self.service_name, &self.provider) {
            if !instance.is_ready() {
                return Err(ServiceError::ServiceUnavailable(
                    "LLM adapter not ready".to_string(),
                ));
            }

            // TODO: Call actual WASM function via WIT bindings
            // For now, return placeholder response
            Ok(format!("LLM response to: {}", message))
        } else {
            Err(ServiceError::ServiceUnavailable(
                "LLM adapter instance not found".to_string(),
            ))
        }
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
