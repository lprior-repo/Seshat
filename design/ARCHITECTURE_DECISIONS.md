# Architectural Decisions

This document summarizes the core Architectural Decision Records (ADRs) that form the foundation of Seshat.

## Core UI Framework (ADR-001)
- **Decision:** Use **Dioxus 0.7** as the cross-platform UI framework.
- **Why:** Single codebase for desktop (`dx run`) and web (`dx serve`). Offers Rust-native, type-safe fine-grained reactive updates without Virtual DOM diffing.

## Cross-Platform Constraints (ADR-006)
- **Decision:** Conditional compilation via WASM + Platform Renderers.
- **Why:** To support native desktop and browser runtimes. 
- **Rule:** `#[cfg(not(target_arch = "wasm32"))]` must isolate the database and CLI from the pure WASM `dioxus/web` target.

## Persistence & Event Sourcing (ADR-003)
- **Decision:** **SQLite + sqlx** in WAL mode.
- **Why:** We utilize append-only event tables (event sourcing) rather than just storing final state. This allows for rich JSON operation logging, time travel, and seamless Two-Way Sync between humans and AI.
- **Conflict Resolution:** We enforce deterministic LWW-Element-Set (Last Writer Wins) CRDT rules mapped to a Hybrid Logical Clock.

## Immutable UI State (ADR-002 & ADR-005)
- **Decision:** Combine Dioxus Signals with immutable data structures from the **`im` and `rpds` crates**.
- **Why:** Using `im::HashMap` allows O(1) cloning due to structural sharing. This makes undo/redo and complex document modifications highly testable and memory-efficient compared to deep cloning a 3,000-node mutable struct.

## Persistent History (ADR-004)
- **Decision:** `rpds` for Undo/Redo stacks.
- **Why:** We maintain full document snapshots capped at a 100-entry memory boundary (FIFO eviction policy). This guarantees perfect inverse restoration without risking complex command pattern mutation bugs.

## Error Handling (ADR-007)
- **Decision:** Absolute reliance on `Result<T, E>` with strict clippy bans on `unwrap` and `panic`.
- **Why:** Forces developers and AI to explicitly model the exact error taxonomy using `thiserror` for the core, and `anyhow` for boundary shells. Ensures zero silent crashes in the UI.