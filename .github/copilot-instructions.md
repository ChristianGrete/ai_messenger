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

### Host-Side Loading

- **Automatic Discovery**: AdapterRegistry loads manifests during adapter initialization
- **Path Support**: Both `latest/` and versioned directory structures supported
- **Default Fallback**: Missing manifests auto-generated from config values (no display_name)
- **Frontend Freedom**: UI decides fallback strategy for missing display_name

### File Locations

```
data_dir/adapters/{service}/{provider}/latest/manifest.json     # Preferred
data_dir/adapters/{service}/{provider}/{version}/manifest.json  # Fallback
```

### Design Philosophy

- **No Auto-Formatting**: display_name stays None if not explicitly set
- **Separation of Concerns**: Backend provides raw data, frontend handles presentation
- **UI Flexibility**: Frontend can implement different fallback strategies

## WASM Runtime Architecture

The WASM adapter system follows a **generic runtime with service-specific traits** pattern for maximum scalability and type safety.

### File Structure

```
src/adapter/
├── mod.rs                    # Public API exports
├── runtime/                  # Generic WASM Runtime
│   ├── mod.rs               # WasmRuntime struct
│   ├── instance.rs          # WasmInstance management
│   └── loader.rs            # WASM module loading
├── services/        # Service-specific implementations
│   ├── mod.rs       # AdapterRegistry with manifest loading
│   ├── llm.rs       # LLM service adapter (MVP: HTTP bypass)
│   └── storage.rs   # Storage service adapter (MVP: HTTP bypass)
└── traits.rs        # Common adapter traits
```

### Design Pattern

**Generic Runtime + Service-Specific Traits**:

- Each service defines its own Rust trait (e.g., `LlmAdapter`, `StorageAdapter`)
- Single `WasmRuntime` loads and manages all WASM modules
- Service-specific wrappers provide type-safe interfaces
- `AdapterRegistry` routes requests to appropriate adapters

**Scalability**: New services require only:

1. New WIT interface in `wit/`
2. New trait in `services/`
3. Registry entry
4. Config schema is already generic

**Benefits**:

- Type-safe service interfaces
- Unified WASM loading/management
- Easy testing via trait mocking
- Config-driven adapter loading

## Implementation Philosophy

**Security & Privacy are non-negotiable top priorities.** Every implementation decision, architectural choice, and feature design must prioritize user data protection and privacy by design. When in doubt between convenience and security, always choose security. World-class privacy standards are a core requirement, not an optional feature.

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

## MVP Implementation Status

### What's Currently Working

- ✅ **Core Server**: Axum-based HTTP server with health endpoint
- ✅ **Config System**: TOML-based configuration with path expansion
- ✅ **CLI Interface**: Serve command with config discovery
- ✅ **Message Endpoint**: `/v1/message/:id` fully functional
- ✅ **Adapter Architecture**: Complete WASM infrastructure (dormant)
- ✅ **Manifest System**: Host-side manifest loading with fallbacks
- ✅ **Ollama Integration**: Direct HTTP calls to local Ollama instance

### Current MVP Limitations

- 🔄 **WASM Bypassed**: Adapters use direct HTTP calls instead of WASM modules
- 🔄 **Single Provider**: Only Ollama LLM adapter implemented
- 🔄 **No Persistence**: No storage adapter active
- 🔄 **Limited Routes**: Only message sending implemented

### WASM Activation Requirements

To activate WASM adapters (when needed):

1. Uncomment WASM loading in `src/adapter/services/llm.rs` and `storage.rs`
2. Implement WIT bindings for adapter communication
3. Build actual WASM modules for each provider
4. Replace direct HTTP calls with WASM function calls

**Design Decision**: MVP intentionally bypasses WASM for rapid development while preserving the complete architecture for future activation.

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

### Routes Structure (Current Implementation Status)

```
src/
  routes/
    health.rs          # GET / (✅ Implemented)
    v1/
      mod.rs           # build & return v1 router (✅ Implemented)
      message/         # POST /v1/message/:id (send message, get AI response)
        mod.rs         # ✅ Fully functional with HTTP bypass to Ollama
      sender/          # 🚧 Partially implemented
        mod.rs         # ✅ Basic structure
        profile.rs     # 🚧 Stub implementation
      recipients/      # � Future implementation
        mod.rs
      recipient/       # � Future implementation (/v1/recipient/:id/*)
        mod.rs
        profile.rs
        picture.rs
      conversations/   # � Future implementation
        mod.rs
      conversation/    # � Future implementation (/v1/conversation/:id/*)
        mod.rs
        # history / pagination handlers
```

### Config System Structure (✅ Complete)

```
src/config/
├── mod.rs           # Public exports
├── creation.rs      # Config file creation
├── defaults.rs      # Default values and constants
├── discovery.rs     # Config file discovery
├── loader.rs        # Config loading with fallbacks
├── paths.rs         # Path expansion utilities
└── schema.rs        # TOML schema with adapter support
```

### WASM Adapter Structure (✅ Infrastructure Complete, MVP Uses HTTP Bypass)

```
src/adapter/
├── mod.rs           # Public API exports
├── manifest.rs      # Adapter manifest system with host-side loading
├── runtime/         # Generic WASM Runtime (complete infrastructure)
│   ├── mod.rs       # WasmRuntime struct
│   ├── instance.rs  # WasmInstance management
│   └── loader.rs    # WASM module loading
├── services/        # Service-specific implementations
│   ├── mod.rs       # AdapterRegistry with manifest loading
│   ├── llm.rs       # LLM service adapter (MVP: HTTP bypass)
│   └── storage.rs   # Storage service adapter (MVP: HTTP bypass)
└── traits.rs        # Common adapter traits
```

## Notes on Route File Layout

- URL hierarchy mirrored directly under `routes/v1/`.
- Mixed plural/singular by design: plural for collections; singular dedicated subtrees for focused item operations.
- `message/` is an action endpoint module (not a collection) → organized as directory for handler logic.
- **`mod.rs` files**: Pure module composition only - build routers, export submodules, NO handler logic.
- Handlers: extract / validate → delegate to service/domain.
- Avoid premature abstraction layers; add only when duplication or complexity emerges.

## Old → New Mapping (For Future Route Implementation)

- `routes/user/*` → `routes/v1/sender/*`
- `routes/contact/:id/*` → `routes/v1/recipient/:id/*` (item handling inside `recipient/` tree)
- `routes/chat/:id` → `routes/v1/message/:id` (send a message, receive a response)
- `routes/<undefined>` → `routes/v1/conversation/:id/*` (chat history loading with pagination)
- `routes/contacts/*` → `routes/v1/recipients/*` (list all available recipients)
- `routes/chats/*` → `routes/v1/conversations/*` (list all existing conversations)

## Base-Path Configuration

**Note**: Base-path support is already implemented in the config system (`server.base_path`).

- The Base-Path is dynamically configurable and defaults to an empty string (`""`).
- It can be set via configuration file: `[server] base_path = "api"`
- The routing logic ensures that the Base-Path is applied globally without altering the existing route definitions.

### Example

- Default behavior (no Base-Path):
  - `/` → Health endpoint
  - `/v1/recipient/:id/name` → Recipient name endpoint
- With `base_path = "api"`:
  - `/api` → Health endpoint
  - `/api/v1/recipient/:id/name` → Recipient name endpoint

## Additional Guardrails for Copilot

- Resource endpoints MUST live under `/v1/*`.
- Health endpoint is ONLY `/` (do not add `/v1/health` unless explicitly requested).
- Base path only via config file (`server.base_path`), not hardcoded `/api` prefixes.
- Do not collapse split route files once justified.
- Keep `main.rs` strictly a dispatcher.
- Preserve separation: routes → services → domain → storage (introduce layers only when needed).
- No speculative directories or placeholder files without near-term implementation.
