// Generic WASM Runtime - module composition only

pub mod instance;
pub mod loader;

pub use instance::WasmInstance;
pub use loader::{LoaderError, ModuleLoader};

use crate::adapter::traits::ServiceError;
use std::collections::HashMap;
use wasmtime::{Config, Engine};

/// Central WASM runtime managing all adapter instances
pub struct WasmRuntime {
    engine: Engine,
    instances: HashMap<String, WasmInstance>,
}

impl WasmRuntime {
    /// Create new WASM runtime with optimized configuration
    pub fn new() -> Result<Self, ServiceError> {
        let mut config = Config::new();

        // Security and performance configuration
        config.wasm_component_model(true);
        config.async_support(true);
        config.consume_fuel(true); // Resource limiting

        let engine = Engine::new(&config).map_err(|e| {
            ServiceError::InitializationFailed(format!("Engine creation failed: {e}"))
        })?;

        Ok(WasmRuntime {
            engine,
            instances: HashMap::new(),
        })
    }

    /// Load a WASM adapter module for a specific service
    pub async fn load_adapter(
        &mut self,
        service: &str,
        module_path: &std::path::Path,
        config_json: &str,
    ) -> Result<(), ServiceError> {
        let loader = ModuleLoader::new(&self.engine);
        let instance = loader.load_module(module_path, config_json).await?;

        let instance_key = format!("{}_{}", service, instance.provider_name());
        self.instances.insert(instance_key, instance);

        Ok(())
    }

    /// Get adapter instance by service and provider
    pub fn get_instance(&self, service: &str, provider: &str) -> Option<&WasmInstance> {
        let key = format!("{}_{}", service, provider);
        self.instances.get(&key)
    }

    /// Get mutable adapter instance by service and provider
    pub fn get_instance_mut(&mut self, service: &str, provider: &str) -> Option<&mut WasmInstance> {
        let key = format!("{}_{}", service, provider);
        self.instances.get_mut(&key)
    }

    /// Shutdown all instances gracefully
    pub async fn shutdown(&mut self) -> Result<(), ServiceError> {
        for (_, instance) in self.instances.drain() {
            instance.shutdown().await?;
        }
        Ok(())
    }

    /// List all loaded adapters
    pub fn list_adapters(&self) -> Vec<(&str, &str, &str)> {
        self.instances
            .iter()
            .map(|(key, instance)| {
                let parts: Vec<&str> = key.split('_').collect();
                let service = parts.first().unwrap_or(&"unknown");
                (*service, instance.provider_name(), instance.version())
            })
            .collect()
    }
}
