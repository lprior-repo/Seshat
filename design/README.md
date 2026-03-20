# Design Documentation

This folder is the central hub for the repository's goals, architectural philosophies, domain boundaries, and engineering rigor.

## 📂 Core Philosophies

- **[GOALS.md](./GOALS.md):** The primary objectives, success metrics, and long-term vision for the project.
- **[WAYS_OF_WORKING.md](./WAYS_OF_WORKING.md):** The standard operating procedures, strict workflows, tooling commands, and day-to-day repository rules.
- **[DOMAIN_DRIVEN_DESIGN.md](./DOMAIN_DRIVEN_DESIGN.md):** The guide to Functional Rust, strict typed boundaries, and the Data → Calc → Actions paradigm.
- **[DIOXUS_07_GUIDE.md](./DIOXUS_07_GUIDE.md):** Detailed constraints and rules for building UI with Dioxus 0.7, Signals, and WASM performance.
- **[ARCHITECTURE_DECISIONS.md](./ARCHITECTURE_DECISIONS.md):** A rolled-up summary of the core ADRs dictating persistence, cross-platform compilation, and state management.

## 📦 Crate & Technology Deep Dives

- **[CRATE_IM_AND_RPDS.md](./CRATE_IM_AND_RPDS.md):** How we use persistent data structures for structural sharing, memory safety, and O(1) Undo/Redo tracking.
- **[CRATE_PETGRAPH.md](./CRATE_PETGRAPH.md):** Using DAG representations for cycle detection and auto-arrange layouts outside the UI thread.
- **[CRATE_SQLX_EVENT_SOURCING.md](./CRATE_SQLX_EVENT_SOURCING.md):** Using SQLite in WAL mode and tokio to build concurrent, lock-free event-sourcing backends.
- **[CRATE_ITERTOOLS_AND_TAP.md](./CRATE_ITERTOOLS_AND_TAP.md):** Functional pipelines. Banning `for` loops in favor of declarative, immutable iteration and suffix logic.
- **[CRATE_THISERROR_AND_ANYHOW.md](./CRATE_THISERROR_AND_ANYHOW.md):** Error boundaries: using `thiserror` for pure domain libraries and `anyhow` for imperative shell context.
- **[CRATE_PROPTEST.md](./CRATE_PROPTEST.md):** Leveraging property-based testing to aggressively fuzz pure core invariants instead of writing happy-path unit tests.

## 🧪 Advanced Testing Strategies

- **[TESTING_E2E_PLAYWRIGHT.md](./TESTING_E2E_PLAYWRIGHT.md):** Asserting the Dioxus web target renders correctly, CSS is applied, and drag/drop JS injections bypass VDOM successfully.
- **[TESTING_CHAOS_AND_CRDT.md](./TESTING_CHAOS_AND_CRDT.md):** Adversarial async test harnesses built to prove that LWW-Element-Set merges always converge under high jitter and concurrent Human+AI load.

---
*Keep these documents updated as the project evolves.*