# Copilot Instructions for ai_messenger

## Purpose

Provide Copilot with the agreed naming and file layout so suggestions stay consistent.
Keep implementation out of scope here; this document is about conventions and structure only.

## Project Vision & Architecture

**ai_messenger** is designed as a **pluggable AI messaging platform** with **WASM-based adapter architecture** for maximum flexibility and provider independence.

### Core Design Principles

- **Provider Independence**: Unified interface for AI service providers (OpenAI, Ollama, etc.) through WASM adapters
- **Storage Flexibility**: Pluggable persistence backends via WASM adapters (JSON files, databases, cloud storage)
- **Optional Encryption**: Configurable encryption layer between core logic and storage for data protection
- **Runtime Isolation**: All adapters run in sandboxed WASM modules for security and stability
- **Hot-Swappable**: Change providers or storage without code changes, just configuration
- **Security & Privacy First**: All implementation decisions prioritize user data protection and privacy by design

### Technical Architecture

```
AI Services ← WASM AI Adapters
     ↓
Core Logic (Rust)
     ↓
[Optional WASM Encryption Layer]
     ↓
WASM Storage Adapters → Physical Storage (JSON files, DB, etc.)
```

**Default Foundation**: Basic JSON file persistence to `data_dir` and local Ollama support as reference implementations.

This vision guides all design decisions: extensibility and security.

## Manifest System Architecture

The adapter manifest system provides metadata for UI consumption and adapter management.

### Manifest Structure

```rust
pub struct AdapterManifest {
    pub name: String,                    // Technical adapter name
    pub version: String,                 // Adapter version
    pub display_name: Option<String>,    // Optional UI-friendly name (no auto-fallback)
    pub models: Option<Vec<String>>,     // Available models (if known)
}
```

### File Locations

```
data_dir/adapters/{service}/{provider}/latest/manifest.json     # Preferred
data_dir/adapters/{service}/{provider}/{version}/manifest.json  # Fallback
```

## WASM Component Model Architecture

The WASM Component Model is **fully implemented and actively used** in the current system.

### WIT Interface Definition

The complete interface is defined in `wit/llm.wit` with the `llm-adapter` world:

- **prepare_request**: Transform generic chat requests into provider-specific HTTP configurations
- **parse_response**: Convert provider responses back into generic chat responses
- **parse_stream_chunk**: Handle streaming responses for real-time communication

### Design Pattern

**WASM Component Model with WIT Interfaces**:

- Each service defines WIT interfaces in `wit/` directory
- Components implement the interface using `wit-bindgen`
- Host uses `wasmtime::component::bindgen` for type-safe integration
- `AdapterRegistry` manages component lifecycle and discovery

## Storage Adapter Architecture

The storage system uses a **key-value abstraction** with pluggable backends via WASM adapters for maximum flexibility.

### Design Philosophy

**Custom Interfaces vs. WASI Standards:**

- **Storage**: Custom key-value interface preferred over WASI filesystem (too low-level)
- **HTTP**: Custom AI-domain interface preferred over WASI HTTP (better host control & AI semantics)
- **Rationale**: Domain-specific abstractions provide better developer experience and host control

### Storage Provider Naming

Storage providers are named by their **data format/technology**, not implementation details:

- ✅ `json` → JSON file storage (filesystem implied)
- ✅ `sqlite` → SQLite database storage
- ✅ `postgres` → PostgreSQL storage
- ✅ `redis` → Redis key-value storage
- ✅ `s3` → Object storage (AWS S3, MinIO, etc.)
- ✅ `memory` → In-memory storage (dev/testing)

**Anti-patterns:**

- ❌ `json-file` → Redundant (JSON implies files)
- ❌ `filesystem` → Too generic
- ❌ `local` → Implementation detail, not format

### Key-Value Interface

All storage adapters implement a unified key-value interface:

```wit
interface storage {
  store: func(key: string, data: list<u8>) -> result<_, error>;
  retrieve: func(key: string) -> result<list<u8>, error>;
  delete: func(key: string) -> result<_, error>;
  exists: func(key: string) -> result<bool, error>;
  list-keys: func(prefix: option<string>) -> result<list<string>, error>;
}
```

### Use Cases & Provider Mapping

- **Development**: `json` (human-readable, version-controllable)
- **Production Local**: `sqlite` (ACID, embedded)
- **Production Distributed**: `postgres` (scalable, robust)
- **Cache Layer**: `redis` (fast, ephemeral)
- **Object Storage**: `s3` (distributed, blob storage)
- **Testing**: `memory` (fast, isolated)

## Implementation Philosophy

**Security & Privacy are non-negotiable top priorities.** Every implementation decision, architectural choice, and feature design must prioritize user data protection and privacy by design. When in doubt between convenience and security, always choose security. World-class privacy standards are a core requirement, not an optional feature.

**No Authentication/Authorization/Roles System**: This platform is intentionally designed as a **local, single-user tool** without any auth/permissions/roles complexity. All data access is assumed to be local and trusted. The security model relies on OS-level permissions and WASM sandboxing, not application-level access control.

## Language & Style

- Code, comments, and docs: US English, concise, idiomatic.
- No auto renames beyond what is listed here.
- **Alphabetical ordering**: Always sort items alphabetically when it makes sense (CLI arguments, imports, struct fields, enum variants, route definitions, etc.) for consistency and readability.

## Naming (final)

- **sender**: the local actor profile and global defaults (singleton in this MVP).
- **recipient**: a persona profile with overrides (per persona).
- **conversation**: a dialogue between sender and one recipient.
- **message**: the smallest unit in a conversation OR a single-shot send endpoint.
- We intentionally mix plural collection segments (`recipients`, `conversations`) with singular item trees (`recipient/:id/*`, `conversation/:id/*`) where it improves clarity.
- `message` is a singular action endpoint: `POST /v1/message/:id`.
- Health endpoint is unversioned at `/` only.

## Routing Prefix and Versioning

We version all resource endpoints under `/v1/*`. Health is intentionally unversioned at `/`.

- All resource routes live under `/v1/*` and in `routes/v1/*`.
- Health endpoint only at `/` (no `/v1/health` unless explicitly introduced later).
- Backwards compatibility: never break `v1`; add `v2/` as a sibling for breaking changes.
- Shared logic (when it appears) must live outside versioned trees; defer creating shared modules until duplication is real.

## Module Organization Principles

- **`mod.rs` files are pure entry points**: Only exports (`pub mod`, `pub use`) and basic module composition (like router building). NO business logic, handlers, or complex implementations.
- Split only when it reduces cognitive load or a file would grow > ~200 LOC.
- **Alphabetical ordering**: Always sort items alphabetically when it makes sense (CLI arguments, imports, struct fields, enum variants, route definitions, module exports, etc.) for consistency and readability.

## Quality Assurance Pipeline

After any code changes, always run the complete QA pipeline to ensure code quality and prevent CI failures:

1. **`cargo check --all-targets --all-features`** - Fast compilation check for syntax errors
2. **`cargo fmt --all`** - Automatic code formatting
3. **`cargo clippy --all-targets --all-features -- -D warnings`** - Lint analysis and best practice enforcement
4. **`cargo test --all-targets --all-features`** - Full test suite validation

This pipeline must pass completely before committing changes. Use `cargo fmt && cargo clippy && cargo test` for efficiency.

## Current Implementation Status

### What's Working

- ✅ **Core Server**: Axum-based HTTP server with health endpoint
- ✅ **Config System**: TOML-based configuration with path expansion
- ✅ **CLI Interface**: Serve command with config discovery
- ✅ **Message Endpoint**: `/v1/message/:id` fully functional
- ✅ **WASM Component Model**: Complete end-to-end WASM Component pipeline
- ✅ **LLM Adapters**: Ollama adapter using WASM Components
- ✅ **Manifest System**: Host-side manifest loading with fallbacks
- ✅ **Adapter Registry**: Dynamic adapter discovery and loading

### Current Scope

- 🔄 **Single Provider**: Only Ollama LLM adapter implemented
- 🔄 **No Storage Adapters**: Infrastructure ready but no implementations
- 🔄 **Limited Routes**: Only message sending implemented

## Code Analysis & Change Protocol

**CRITICAL**: Before making ANY code changes, always follow this protocol to maintain codebase integrity:

### Pre-Change Analysis (MANDATORY)

1. **Analyze Existing Codebase**: Use semantic search, file search, and read existing files to understand current implementation
2. **Check for Existing Implementations**: Verify if requested functionality already exists before creating duplicates
3. **Understand Dependencies**: Review imports, exports, and module relationships to avoid conflicts
4. **Assess Impact Scope**: Determine minimal change set required - avoid unnecessary modifications

### Change Execution Principles

- **Minimal Invasive**: Make only the smallest changes necessary to achieve the goal
- **Incremental**: Break large changes into small, testable steps
- **Conservative**: Preserve existing working code unless explicitly asked to refactor
- **Verification**: Run QA pipeline after each change to ensure stability

### Anti-Patterns to Avoid

- ❌ **Assumption-Based Coding**: Don't assume what exists - verify first
- ❌ **Scope Creep**: Don't implement unrequested features "while you're at it"
- ❌ **Destructive Refactoring**: Don't restructure unless explicitly requested
- ❌ **Duplicate Dependencies**: Always check existing imports before adding new ones

**Remember**: Stability and incremental progress over ambitious changes that break the build.

## Project File Structure

### Routes Structure

```
src/routes/
├── health.rs          # GET / (✅ Implemented)
└── v1/
    ├── mod.rs         # v1 router (✅ Implemented)
    ├── message/       # POST /v1/message/:id (✅ Fully functional)
    │   └── mod.rs
    ├── sender/        # 🚧 Partially implemented
    │   ├── mod.rs
    │   └── profile.rs
    ├── recipients/    # � Future implementation
    ├── recipient/     # � Future implementation (/v1/recipient/:id/*)
    ├── conversations/ # � Future implementation
    └── conversation/  # � Future implementation (/v1/conversation/:id/*)
```

### WASM Adapter Structure

```
src/adapter/
├── mod.rs           # Public API exports
├── manifest.rs      # Adapter manifest system
├── wit.rs           # Host-side WIT bindings (✅ Active)
├── runtime/         # WASM Runtime (✅ Functional)
│   ├── mod.rs       # WasmRuntime struct
│   ├── instance.rs  # WasmInstance management
│   └── loader.rs    # WASM module loading
├── services/        # Service-specific implementations
│   ├── mod.rs       # AdapterRegistry
│   ├── llm.rs       # LLM service adapter (✅ WASM integrated)
│   └── storage.rs   # Storage service adapter (infrastructure ready)
└── traits.rs        # Common adapter traits

adapters/llm/ollama/ # ✅ Complete WASM Component
├── src/
│   ├── lib.rs       # WASM Component with WIT bindings
│   └── bindings.rs  # Generated WIT bindings
├── scripts/
│   └── build.sh     # WASM Component build pipeline
├── wit/
│   └── world.wit    # Component interface definition
└── Cargo.toml       # cargo-component configuration

wit/llm.wit         # ✅ Master WIT interface definition
```

## Route File Layout Notes

- URL hierarchy mirrored directly under `routes/v1/`
- Mixed plural/singular by design: plural for collections; singular for item operations
- `message/` is an action endpoint module (not a collection)
- **`mod.rs` files**: Pure module composition only - NO handler logic
- Handlers: extract / validate → delegate to service/domain

## Base-Path Configuration

Base-path support is implemented in the config system (`server.base_path`). Defaults to empty string.

- Set via config: `[server] base_path = "api"`
- Example: `/api/v1/message/:id` instead of `/v1/message/:id`

## Additional Guardrails for Copilot

- Resource endpoints MUST live under `/v1/*`.
- Health endpoint is ONLY `/` (do not add `/v1/health` unless explicitly requested).
- Base path only via config file (`server.base_path`), not hardcoded `/api` prefixes.
- Do not collapse split route files once justified.
- Keep `main.rs` strictly a dispatcher.
- Preserve separation: routes → services → domain → storage (introduce layers only when needed).
- No speculative directories or placeholder files without near-term implementation.
