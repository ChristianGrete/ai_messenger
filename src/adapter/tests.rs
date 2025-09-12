#[cfg(test)]
mod adapter_tests {
    use crate::adapter::traits::ModelInfo;
    use crate::adapter::{AdapterRegistry, ServiceError, WasmRuntime};

    #[tokio::test]
    async fn test_adapter_registry_creation() {
        let registry = AdapterRegistry::new().await;
        assert!(registry.is_ok());
    }

    #[tokio::test]
    async fn test_wasm_runtime_creation() {
        let runtime = WasmRuntime::new();
        assert!(runtime.is_ok());
    }

    #[test]
    fn test_service_error_types() {
        let error = ServiceError::InitializationFailed("test".to_string());
        assert!(error.to_string().contains("test"));

        let error = ServiceError::ExecutionError("exec".to_string());
        assert!(error.to_string().contains("exec"));
    }

    #[test]
    fn test_model_info_display() {
        let model_info = ModelInfo {
            name: "test-model".to_string(),
            version: "1.0".to_string(),
            context_length: Some(4096),
            parameters: Some("7B".to_string()),
        };

        let display = format!("{}", model_info);
        assert!(display.contains("test-model"));
        assert!(display.contains("1.0"));
    }

    #[tokio::test]
    async fn test_registry_empty_adapters() {
        let registry = AdapterRegistry::new().await.unwrap();

        // Should have no adapters initially
        assert!(registry.get_default_llm_adapter().is_none());
        assert!(registry.get_default_storage_adapter().is_none());
        assert!(registry.get_llm_adapter("ollama").is_none());
        assert!(registry.get_storage_adapter("json").is_none());
    }
}
