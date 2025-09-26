// Host-side WIT bindings for WASM component communication

use wasmtime::component::bindgen;

// LLM adapter bindings
pub mod llm {
    use wasmtime::component::bindgen;

    bindgen!({
        world: "llm-adapter",
        path: "wit/llm",
        async: true,
    });
}

// Storage adapter bindings
pub mod storage {
    use wasmtime::component::bindgen;

    bindgen!({
        world: "storage-adapter",
        path: "wit/storage",
        async: true,
    });
}

pub use llm::exports::ai_messenger::llm::llm as llm_interface;
pub use storage::exports::ai_messenger::storage::storage as storage_interface;

// Host-provided capabilities for storage adapters
pub use storage::ai_messenger::storage::host_capabilities::Host as StorageHostCapabilities;

// Type aliases for better ergonomics
pub mod llm_types {
    pub use crate::adapter::wit::llm::ai_messenger::llm::types::{
        ChatRequest, ChatResponse, FinishReason, HttpConfig, HttpResponse, Message, Role,
        StreamChunk, Usage,
    };
}

pub mod storage_types {
    pub use crate::adapter::wit::storage::ai_messenger::storage::types::{
        HttpConfig as StorageHttpConfig, HttpResponse as StorageHttpResponse, StorageError,
    };
}
