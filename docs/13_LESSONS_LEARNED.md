# Lessons Learned

This document tracks critical technical constraints, anti-patterns, and architectural rules discovered during development. 
Before attempting any major feature implementation, consult these lessons to avoid known pitfalls.

## 1. Dioxus WASM Build Constraints (CRITICAL)

When running `dx build --platform web` or building for `wasm32-unknown-unknown`, the build will fatally panic if backend networking or database logic leaks into the WASM client.

- **NEVER** include `sqlx`, `tokio`, `mio`, `rusqlite`, or `reqwest` (with default TLS) in the `wasm32-unknown-unknown` target.
- **ALWAYS** wrap server/database dependencies in `Cargo.toml` with `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`.
- **NEVER** set `default = ["desktop", "server"]` or enable `fullstack` by default in `Cargo.toml`. You must use `default = ["web"]` to ensure `dx build` does not accidentally pull in `dioxus-server` and `tokio`.
- **ALWAYS** wrap domain functions that take `SqlitePool` with `#[cfg(not(target_arch = "wasm32"))]`.

## 2. Strict TDD Enforcement
- Do not bypass `tdd-guard`. All feature code and refactors require failing tests to be written and validated via `tdd-guard-rust` first.
- **NO Subagents**: You must NEVER use subagent development (e.g. `go-skill`, `Task` tool, or other autonomous agents) to write code. You must perform the development directly in this interactive session so that TDD Guard constraints are strictly enforced.
- Do not over-implement. Write only the minimum code necessary to make the failing test pass.