use crate::adapter::runtime::WasmRuntime;
use crate::adapter::traits::{AdapterService, ServiceError, StorageAdapter};
use crate::config::schema::ServiceAdapterConfig;
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Storage adapter wrapper providing typed interface to WASM instances
pub struct StorageAdapterWrapper {
    runtime: Arc<RwLock<WasmRuntime>>,
    provider: String,
    version: String,
    service_name: String,
}

impl StorageAdapterWrapper {
    /// Create new storage adapter wrapper
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

        Ok(StorageAdapterWrapper {
            runtime: runtime.clone(),
            provider: config.provider.clone(),
            version: config.version.clone(),
            service_name: service_name.to_string(),
        })
    }
}

#[async_trait]
impl AdapterService for StorageAdapterWrapper {
    fn service_name(&self) -> &'static str {
        "storage"
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
#[async_trait]
impl StorageAdapter for StorageAdapterWrapper {
    async fn store(&mut self, key: &str, data: &[u8]) -> Result<(), ServiceError> {
        let runtime = self.runtime.read().await;

        if let Some(instance) = runtime.get_instance(&self.service_name, &self.provider) {
            if !instance.is_ready() {
                return Err(ServiceError::ServiceUnavailable(
                    "Storage adapter not ready".to_string(),
                ));
            }

            // TODO: Call actual WASM function via WIT bindings
            // For now, simulate successful storage
            tracing::debug!("Storing {} bytes with key: {}", data.len(), key);
            Ok(())
        } else {
            Err(ServiceError::ServiceUnavailable(
                "Storage adapter instance not found".to_string(),
            ))
        }
    }

    async fn retrieve(&self, key: &str) -> Result<Vec<u8>, ServiceError> {
        let runtime = self.runtime.read().await;

        if let Some(instance) = runtime.get_instance(&self.service_name, &self.provider) {
            if !instance.is_ready() {
                return Err(ServiceError::ServiceUnavailable(
                    "Storage adapter not ready".to_string(),
                ));
            }

            // TODO: Call actual WASM function via WIT bindings
            // For now, return placeholder data
            Ok(format!("placeholder_data_for_{}", key).into_bytes())
        } else {
            Err(ServiceError::ServiceUnavailable(
                "Storage adapter instance not found".to_string(),
            ))
        }
    }

    async fn delete(&mut self, key: &str) -> Result<(), ServiceError> {
        let runtime = self.runtime.read().await;

        if let Some(instance) = runtime.get_instance(&self.service_name, &self.provider) {
            if !instance.is_ready() {
                return Err(ServiceError::ServiceUnavailable(
                    "Storage adapter not ready".to_string(),
                ));
            }

            // TODO: Call actual WASM function via WIT bindings
            tracing::debug!("Deleting key: {}", key);
            Ok(())
        } else {
            Err(ServiceError::ServiceUnavailable(
                "Storage adapter instance not found".to_string(),
            ))
        }
    }

    async fn exists(&self, key: &str) -> Result<bool, ServiceError> {
        let runtime = self.runtime.read().await;

        if let Some(instance) = runtime.get_instance(&self.service_name, &self.provider) {
            if !instance.is_ready() {
                return Err(ServiceError::ServiceUnavailable(
                    "Storage adapter not ready".to_string(),
                ));
            }

            // TODO: Call actual WASM function via WIT bindings
            // For now, simulate existence check
            Ok(!key.is_empty())
        } else {
            Err(ServiceError::ServiceUnavailable(
                "Storage adapter instance not found".to_string(),
            ))
        }
    }

    async fn list_keys(&self, prefix: Option<&str>) -> Result<Vec<String>, ServiceError> {
        let runtime = self.runtime.read().await;

        if let Some(instance) = runtime.get_instance(&self.service_name, &self.provider) {
            if !instance.is_ready() {
                return Err(ServiceError::ServiceUnavailable(
                    "Storage adapter not ready".to_string(),
                ));
            }

            // TODO: Call actual WASM function via WIT bindings
            // For now, return placeholder keys
            let keys = match prefix {
                Some(p) => vec![format!("{}_key1", p), format!("{}_key2", p)],
                None => vec!["key1".to_string(), "key2".to_string()],
            };

            Ok(keys)
        } else {
            Err(ServiceError::ServiceUnavailable(
                "Storage adapter instance not found".to_string(),
            ))
        }
    }
}
