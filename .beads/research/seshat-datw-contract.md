# Architectural Contract: Phase 1 Dioxus Store (seshat-datw)

**Role:** Arch-Design-QA (The Relentless Interrogator)
**Feature:** Centralizing 15 Disconnected Signals into a Unified `AppState`

---

## Phase 1: EARS (Eliminate Requirements Ambiguity)

### Ubiquitous Language & Event Triggers
- **Ubiquitous:** THE SYSTEM SHALL instantiate a single centralized `AppState` struct, grouping 15 previously global, disconnected `Signal` and `use_context_provider` instances into strictly typed logical domains (e.g., `DocumentState`, `UiState`, `InteractionState`).
- **Event-Driven:** WHEN the application `App()` component initializes, THE SYSTEM SHALL instantiate this unified `AppState` and inject it exactly once via `use_context_provider`.
- **State-Driven:** WHILE a child component is rendering, THE SYSTEM SHALL provide access to the centralized state via `use_context::<AppState>()`.
- **Unwanted:** IF a piece of state is purely derived from `DiagramDocument` (e.g., `ToolbarStats`), THE SYSTEM SHALL NOT store it as an independent mutable `Signal` that requires manual synchronization. It MUST be computed dynamically or memoized via `Memo<T>`.

---

## Phase 2: KIRK Contracts (Domain Modeling)

### Preconditions
- The environment must strictly adhere to the `wasm32-unknown-unknown` constraint rules (`tokio`, `sqlx`, `reqwest` are omitted or feature-gated).
- Dioxus Context must be uninitialized prior to `App()` providing the unified `AppState`.

### Invariants & Unrepresentable States (CRITICAL)
Currently, `diagram_tool/src/app/mod.rs` injects 15 signals, including semantically ambiguous primitives (e.g., `Signal<bool>`, `Signal<(f64, f64)>`). This creates a massive hole in the domain model. We are making the following illegal states **unrepresentable through Rust's type system**:

1. **The "Mystery Meat" State Vector:**
   - *Illegal State:* Injecting `Signal<bool>` or `Signal<u64>` globally allows any component to write to an untyped variable, clobbering other state.
   - *Unrepresentable By:* Forcing all fields in `AppState` to use strong, domain-specific type names (e.g., `is_dragging: Signal<bool>`, `revision_counter: Signal<u64>`, `pan_zoom_offset: Signal<(f64, f64)>`).

2. **The Desynchronized Derivative:**
   - *Illegal State:* `ToolbarStats` is manually synced via `use_effect`. It is possible for `DiagramDocument` to update but `ToolbarStats` to be stale if the effect fails or lags.
   - *Unrepresentable By:* Removing `ToolbarStats` from the list of mutable global state entirely. It must become a derived `Memo<ToolbarStats>` (or computed locally) directly linked to `DiagramDocument` within `AppState`.

3. **The Disjoint Document History:**
   - *Illegal State:* `DiagramDocument` and `History` are separate signals. Mutating `DiagramDocument` without pushing to `History` is a valid compilation state but a semantic bug that destroys undo/redo.
   - *Unrepresentable By:* Grouping them under a `DocumentState` struct and (in later phases) funneling updates exclusively through a `Calc` command (Data -> Calc -> Actions) that outputs a unified `(Document, History)` tuple update.

---

## Phase 3: Inversion (Error Taxonomy & Failure Modes)

**Name every single way this centralization can fail:**
- **`ContextMissingError` (Panic):** A child component attempts to access `use_context::<AppState>()` outside the `App` provider tree.
- **`DesyncError` (UI Freeze):** A component holds onto a stale read lock of a `Signal` inside `AppState` while another attempts to write. Long-lived read references are fatal in Dioxus reactive state.
- **`WasmTargetViolation` (Compilation Failure):** `AppState` inadvertently imports backend dependencies instead of delegating to the Action shell.
- **`OverSubscriptionError` (Performance Collapse):** Child components reading from the entire struct rather than specific signals, causing massive re-renders.

---

## Phase 4: Second-Order Consequence Tracing

**"If we centralize this into a single Store, what happens to our concurrent throughput?"**
- **Blast Radius:** If `AppState` groups 15 signals into a single struct injected via context, any component calling `use_context::<AppState>()` gets access to ALL state. If implemented naively, this leads to a massive dependency graph where panning the canvas causes the sidebar to re-render.
- **Mitigation Guarantee:** The contract strictly demands that `AppState` is a **struct containing `Signal<T>` fields** (e.g., `AppState { doc: Signal<DiagramDocument>, history: Signal<History> }`), NOT a **`Signal` containing a struct** (`Signal<AppState>`). Because the fields themselves are signals, a component only subscribes to a specific piece of state when it calls `.read()` on that field. Thus, structural centralization does not trigger global DOM re-renders.

---

## Phase 5: Pre-Mortem (The 3 AM Red Build)

**It is 3 months from now. We launched this, and production just went down. The UI is freezing completely when users drag a box. Why did it happen?**
- **Root Cause:** A developer violated the struct-of-signals contract and wrapped `AppState` in a single `Signal<AppState>`, forcing Dioxus to diff the entire DOM tree (including 3,000 SVG nodes) on every single mouse movement payload updates.
- **Prevention/Telemetry:** Code review guidelines and CI constraints must guarantee that the root `App()` component only exposes field-level signals via context, or that `#[derive(Store)]` (if utilized) generates isolated field-level reactivity. The application's core `dx serve` performance logs will catch frame drops during the `test_e2e_pipeline` checks.

---
**Handoff:** The architecture is conceptually hardened. The unrepresentable states are defined. Execute implementation tasks natively.