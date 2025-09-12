use crate::adapter::runtime::instance::WasmInstance;
use crate::adapter::traits::ServiceError;
use std::path::Path;
use thiserror::Error;
use wasmtime::{Engine, component::Component};

#[derive(Error, Debug)]
pub enum LoaderError {
    #[error("Failed to read WASM module file: {0}")]
    FileReadError(#[from] std::io::Error),
    #[error("Failed to compile WASM module: {0}")]
    CompilationError(String),
    #[error("Invalid WASM component: {0}")]
    InvalidComponent(String),
}

impl From<LoaderError> for ServiceError {
    fn from(err: LoaderError) -> Self {
        ServiceError::InitializationFailed(err.to_string())
    }
}

/// WASM module loader handling component compilation and validation
pub struct ModuleLoader<'a> {
    engine: &'a Engine,
}

impl<'a> ModuleLoader<'a> {
    /// Create new module loader
    pub fn new(engine: &'a Engine) -> Self {
        ModuleLoader { engine }
    }

    /// Load and compile WASM component from file
    pub async fn load_module(
        &self,
        module_path: &Path,
        config_json: &str,
    ) -> Result<WasmInstance, ServiceError> {
        // Validate file exists
        if !module_path.exists() {
            return Err(ServiceError::InitializationFailed(format!(
                "WASM module not found: {}",
                module_path.display()
            )));
        }

        // Read WASM bytes
        let wasm_bytes = tokio::fs::read(module_path)
            .await
            .map_err(LoaderError::FileReadError)?;

        // Compile component
        let component = Component::new(self.engine, &wasm_bytes)
            .map_err(|e| LoaderError::CompilationError(e.to_string()))?;

        // Extract metadata from file path
        let (provider_name, version) = self.extract_metadata(module_path)?;

        // Create instance
        let mut instance = WasmInstance::new(
            self.engine,
            component,
            provider_name,
            version,
            config_json.to_string(),
        )?;

        // Initialize the instance
        instance.initialize().await?;

        Ok(instance)
    }

    /// Extract provider name and version from module path
    /// Expected path: data/adapters/{service}/{provider}/{version}/adapter.wasm
    fn extract_metadata(&self, module_path: &Path) -> Result<(String, String), ServiceError> {
        const PROVIDER_OFFSET: usize = 2;
        const VERSION_OFFSET: usize = 3;

        let path_str = module_path.to_string_lossy();
        let parts: Vec<&str> = path_str.split('/').collect();

        // Find the adapters directory and extract provider/version
        if let Some(adapters_index) = parts.iter().position(|&part| part == "adapters") {
            let provider = parts.get(adapters_index + PROVIDER_OFFSET).ok_or_else(|| {
                ServiceError::InvalidConfig("Cannot extract provider from path".to_string())
            })?;
            let version = parts.get(adapters_index + VERSION_OFFSET).ok_or_else(|| {
                ServiceError::InvalidConfig("Cannot extract version from path".to_string())
            })?;

            Ok((provider.to_string(), version.to_string()))
        } else {
            Err(ServiceError::InvalidConfig(
                "Invalid adapter path format".to_string(),
            ))
        }
    }

    /// Validate WASM component exports (future enhancement)
    pub async fn validate_component(&self, _component: &Component) -> Result<(), ServiceError> {
        // TODO: Validate that component exports expected WIT interface
        // This will be implemented once we have proper WIT bindings
        Ok(())
    }
}
