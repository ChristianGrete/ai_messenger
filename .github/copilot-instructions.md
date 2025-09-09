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

1. **`cargo check`** - Fast compilation check for syntax errors
2. **`cargo fmt`** - Automatic code formatting
3. **`cargo clippy`** - Lint analysis and best practice enforcement
4. **`cargo test`** - Full test suite validation

This pipeline must pass completely before committing changes. Use `cargo fmt && cargo clippy && cargo test` for efficiency.

## Project File Structure (Routes - To Be Implemented)

Routes structure to follow when implementing the server:

```
src/
  routes/
    health.rs          # GET /
    v1/
      mod.rs           # build & return v1 router
      sender/
        mod.rs
        profile.rs
        picture.rs
      recipients/       # collection endpoints (list/create) when implemented
        mod.rs
      recipient/        # item subtree (/v1/recipient/:id/*)
        mod.rs
        profile.rs
        picture.rs
      conversations/    # collection endpoints (list/create) when implemented
        mod.rs
      conversation/     # item subtree (/v1/conversation/:id/*)
        mod.rs
        # history / pagination handlers added later
      message.rs        # POST /v1/message/:id (send message, get AI response)
```

## Notes on Route File Layout

- URL hierarchy mirrored directly under `routes/v1/`.
- Mixed plural/singular by design: plural for collections; singular dedicated subtrees for focused item operations.
- `message.rs` is an action endpoint (not a collection) → stays singular.
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
