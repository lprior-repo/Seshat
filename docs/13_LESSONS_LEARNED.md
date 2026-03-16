# Lessons Learned

This document tracks critical technical constraints, anti-patterns, and architectural rules discovered during development. 
Before attempting any major feature implementation, consult these lessons to avoid known pitfalls.

## 1. Dioxus WASM Build Constraints (CRITICAL)

When running `dx build --platform web` or building for `wasm32-unknown-unknown`, the build will fatally panic if backend networking or database logic leaks into the WASM client.

- **NEVER** include `sqlx`, `tokio`, `mio`, `rusqlite`, or `reqwest` (with default TLS) in the `wasm32-unknown-unknown` target.
- **ALWAYS** wrap server/database dependencies in `Cargo.toml` with `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`.
- **NEVER** set `default = ["desktop", "server"]` or enable `fullstack` by default in `Cargo.toml`. You must use `default = ["web"]` to ensure `dx build` does not accidentally pull in `dioxus-server` and `tokio`.
- **ALWAYS** wrap domain functions that take `SqlitePool` with `#[cfg(not(target_arch = "wasm32"))]`.

## 2. Undo History Constraints (EXPLICIT)
- **Memory budget**: 512 MB maximum
- **Maximum snapshots**: 100
- **eviction policy**: FIFO (drop oldest)
- **When evicting**: The oldest snapshot is silently discarded
- **User notification**: None (eviction is silent to avoid UI disruption)

## 3. Conflict Resolution (CRDT)
- **Algorithm**: CRDT (Conflict-free Replicated Data Types)
- **Why**: Automatic merge, human-friendly, AI-friendly
- **When concurrent edits conflict**: CRDT automatically merges changes
- **Clock skew handling**: Local HLC is adjusted during merge
- **No data loss**: Both changes preserved

## 4. Serialization Format
- **Format**: JSON
- **Why**: Schema matches, human-readable, AI-friendly
- **payload field**: JSON-encoded operation data
## 5. Strict TDD Enforcement
- **tdd-guard mandatory**: All feature code requires failing tests first
- **test command**: `cargo nextest run 2>&1 | tdd-guard-rust --project-root . --passthrough`
- **NO implementation without tests**: Code will be rejected
- **write only minimum code to make tests pass**
## 6. NO Subagent Development
- **NEVER** use `go-skill`, `Task` tool, or other autonomous agents
- **All development must happen in interactive session
- **Why**: TDD Guard constraints must be enforced in real-time
## 7. Verification Tools (CRITICAL)
- **Kani**: Formal verification for state machines and geometry
- **Black Hat Reviewer**: Contract Parity, Functional Rust Big 6, Strict DDD
- **Truth Serum**: AI hallucination detection, laziness detection
- **Red Queen**: Adversarial testing (boundary values, malformed inputs)
