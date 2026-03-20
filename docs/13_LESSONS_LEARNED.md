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

## 8. Dioxus 0.7 + Tailwind CSS v4
- **Required Setup**: Tailwind v4 now requires explicit manual compilation when used with Dioxus 0.7. `npm init` and `npm install @tailwindcss/cli` must be run.
- **Config**: Define `input.css` with `@import "tailwindcss"; @source "./src/**/*.{rs,html,css}";`.
- **Injection**: Include `document::Stylesheet { href: asset!("/assets/tailwind.css") }` inside your `ThemeProvider` or root `App` component.
- **Assets**: Update `Dioxus.toml` to explicitly point `asset_dir = "assets"`.
- **Runtime**: Must run the Tailwind watcher (`npx @tailwindcss/cli -i ./input.css -o ./assets/tailwind.css --watch`) alongside `dx serve`.

## 9. Dioxus UI & Layout Gotchas
- **Scrollbar Overflow**: Applying `overflow-x-auto` to a `h-14` flex row without `shrink-0` inside a flex container can cause severe layout cascading issues, wiping out entire toolbars and rendering huge scrollbars on the main canvas.
- **Heavy DOM Performance**: Dioxus 0.7 handles massive DOM trees well (rendering 2,400+ SVG components seamlessly). Do not unnecessarily build manual pagination limits if native scrolling performs perfectly.

## 10. Dioxus Agent RS (Webdriver)
- To properly debug Dioxus WASM visually via agents, use `dioxus-agent-rs`.
- It supports a persistent `repl` mode (via `rustyline` and `shlex`) allowing rapid interactive debugging without spinning up/tearing down the chromedriver session for each command.
