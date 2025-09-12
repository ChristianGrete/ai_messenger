// Host-side WIT bindings for WASM component communication

use wasmtime::component::bindgen;

bindgen!({
    world: "llm-adapter",
    path: "wit",
    async: true
});

pub use exports::ai_messenger::llm::llm as llm_interface;
