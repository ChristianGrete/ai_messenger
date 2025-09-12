// Service-specific adapter implementations

pub mod llm;
pub mod storage;
// Future services:
// pub mod stt;
// pub mod tts;

use crate::adapter::runtime::WasmRuntime;
use crate::adapter::services::{llm::LlmAdapterWrapper, storage::StorageAdapterWrapper};
use crate::adapter::traits::{AdapterService, ServiceError};
use crate::config::schema::Config;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Central registry managing all service adapters
pub struct AdapterRegistry {
    runtime: Arc<RwLock<WasmRuntime>>,
    llm_adapters: HashMap<String, LlmAdapterWrapper>,
    storage_adapters: HashMap<String, StorageAdapterWrapper>,
}

impl AdapterRegistry {
    /// Create new adapter registry
    pub async fn new() -> Result<Self, ServiceError> {
        let runtime = WasmRuntime::new()?;

        Ok(AdapterRegistry {
            runtime: Arc::new(RwLock::new(runtime)),
            llm_adapters: HashMap::new(),
            storage_adapters: HashMap::new(),
        })
    }

    /// Initialize all adapters from configuration
    pub async fn initialize_from_config(
        &mut self,
        config: &Config,
        data_dir: &Path,
    ) -> Result<(), ServiceError> {
        for (service_name, service_config) in &config.adapters.services {
            match service_name.as_str() {
                "llm" => {
                    let adapter = llm::LlmAdapterWrapper::new(
                        &self.runtime,
                        service_config,
                        data_dir,
                        service_name,
                    )
                    .await?;

                    self.llm_adapters
                        .insert(service_config.provider.clone(), adapter);
                }
                "storage" => {
                    let adapter = storage::StorageAdapterWrapper::new(
                        &self.runtime,
                        service_config,
                        data_dir,
                        service_name,
                    )
                    .await?;

                    self.storage_adapters
                        .insert(service_config.provider.clone(), adapter);
                }
                _ => {
                    tracing::warn!("Unknown service type: {}", service_name);
                }
            }
        }

        Ok(())
    }

    /// Get LLM adapter by provider name
    pub fn get_llm_adapter(&self, provider: &str) -> Option<&LlmAdapterWrapper> {
        self.llm_adapters.get(provider)
    }

    /// Get mutable LLM adapter by provider name
    pub fn get_llm_adapter_mut(&mut self, provider: &str) -> Option<&mut LlmAdapterWrapper> {
        self.llm_adapters.get_mut(provider)
    }

    /// Get storage adapter by provider name
    pub fn get_storage_adapter(&self, provider: &str) -> Option<&StorageAdapterWrapper> {
        self.storage_adapters.get(provider)
    }

    /// Get mutable storage adapter by provider name
    pub fn get_storage_adapter_mut(
        &mut self,
        provider: &str,
    ) -> Option<&mut StorageAdapterWrapper> {
        self.storage_adapters.get_mut(provider)
    }

    /// Get default LLM adapter (first available)
    pub fn get_default_llm_adapter(&self) -> Option<&LlmAdapterWrapper> {
        self.llm_adapters.values().next()
    }

    /// Get default storage adapter (first available)
    pub fn get_default_storage_adapter(&self) -> Option<&StorageAdapterWrapper> {
        self.storage_adapters.values().next()
    }
    /// List all loaded adapters
    pub async fn list_adapters(&self) -> Vec<(String, String, String, String)> {
        let runtime = self.runtime.read().await;
        runtime
            .list_adapters()
            .into_iter()
            .map(|(service, provider, version)| {
                (
                    service.to_string(),
                    provider.to_string(),
                    version.to_string(),
                    "ready".to_string(),
                )
            })
            .collect()
    }

    /// Graceful shutdown of all adapters
    pub async fn shutdown(&mut self) -> Result<(), ServiceError> {
        // Shutdown service adapters
        for (_, mut adapter) in self.llm_adapters.drain() {
            adapter.shutdown().await?;
        }

        for (_, mut adapter) in self.storage_adapters.drain() {
            adapter.shutdown().await?;
        }

        // Shutdown runtime
        let mut runtime = self.runtime.write().await;
        runtime.shutdown().await?;

        Ok(())
    }
}
