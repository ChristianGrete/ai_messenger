use crate::adapter::traits::ServiceError;
use wasmtime::{Store, component::Component};

/// WASM instance wrapper providing lifecycle management
pub struct WasmInstance {
    store: Store<InstanceState>,
    #[allow(dead_code)] // Used in future WIT bindings implementation
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

        // TODO: Instantiate component and call initialization function
        // This will be implemented once we have WIT bindings

        self.store.data_mut().is_initialized = true;
        self.is_ready = true;

        Ok(())
    }

    /// Execute a function call on the WASM instance
    pub async fn call_function(
        &mut self,
        _function_name: &str,
        _args: &[u8],
    ) -> Result<Vec<u8>, ServiceError> {
        if !self.is_ready {
            return Err(ServiceError::ServiceUnavailable(
                "Instance not initialized".to_string(),
            ));
        }

        // Add fuel for execution
        self.store
            .set_fuel(100_000)
            .map_err(|e| ServiceError::ExecutionError(format!("Fuel setting failed: {e}")))?;

        // TODO: Implement actual function calling via WIT bindings
        // For now, return placeholder
        Ok(b"placeholder_response".to_vec())
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
