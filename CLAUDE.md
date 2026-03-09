# Seshat Project Guidelines

This project strictly follows specific engineering practices and toolchains. As an AI assistant, you must adhere to the following rules:

## Core Stack & Tools
- **Codanna**: Always use codanna for code intelligence and semantic search (`codanna --config .codanna/settings.toml serve --watch`).
- **Moon**: The primary build system. Use `moon run <task>` instead of raw `cargo` commands when possible.
- **Beads (`bd`)**: Use `bd` for ALL issue tracking. Do NOT use markdown TODOs. 
- **Jujutsu (`jj`)**: Use `jj` for version control alongside `git`.

## Functional Rust
- Enforce the **Data → Calculations → Actions** pattern.
- **Zero panics, zero unwrap, zero mut** by default in source code.
- Always use `Result<T, E>` for errors.
- Ensure strict compliance with `clippy-source` for flawless code.

## Landing the Plane (Full Moon Landing)
When ending a session or completing a feature, you **MUST execute a "Full Moon Landing"**.

A session is NOT complete until all these steps are done:
1. **Run Quality Gates**: You must run `moon run :ci-source` and ensure it passes completely.
2. **File Issues**: Use `bd` to track any remaining or discovered work.
3. **Push to Remote**: 
   ```bash
   git pull --rebase
   bd sync
   git push
   git status # MUST show "up to date with origin"
   ```
4. **Never stop before pushing**: Do not leave work stranded locally.

## Dioxus WASM Build Constraints (CRITICAL)
When running `dx build --platform web` or building for `wasm32-unknown-unknown`, the build will fatally panic if backend networking or database logic leaks into the WASM client.
- **NEVER** include `sqlx`, `tokio`, `mio`, `rusqlite`, or `reqwest` (with default TLS) in the `wasm32-unknown-unknown` target.
- **ALWAYS** wrap server/database dependencies in `Cargo.toml` with `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`.
- **NEVER** set `default = ["desktop", "server"]` or enable `fullstack` by default in `Cargo.toml`. You must use `default = ["web"]` to ensure `dx build` does not accidentally pull in `dioxus-server` and `tokio`.
- **ALWAYS** wrap domain functions that take `SqlitePool` with `#[cfg(not(target_arch = "wasm32"))]`.
