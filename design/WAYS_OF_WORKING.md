# Ways of Working: Gory Details & Engineering Rigor

This document defines the strict standard operating procedures, mandatory architectural constraints, tooling requirements, and day-to-day workflow instructions for Seshat. Adhering to these principles ensures zero-defect delivery and absolute state consistency in a real-time multiplayer environment.

---

## 🏗️ 1. Functional Rust (Data → Calc → Actions)

We strictly follow the **Functional Core, Imperative Shell** pattern. This enforces a hard boundary between state mutation, network calls, and complex logic, making our core perfectly testable and side-effect free.

### The Three Tiers

1. **Data (Immutable Domain Models)**
   - Located in `diagram_tool/src/models/`.
   - Inert, serializable, comparable structs and enums. 
   - We utilize newtypes to prevent primitive obsession (e.g., `CustomerId(String)`, `OrderedFloat`).
   - *Rule*: Make illegal states unrepresentable using enums rather than loose booleans. If a node cannot be shipped and drafted at the same time, make it an enum: `enum NodeState { Draft, Validated }`.

2. **Calc (Pure Functions)**
   - Located in `diagram_tool/src/core/` and `diagram_tool/src/mutation/`.
   - Time-independent, referentially transparent. Takes `Data` and returns `Result<Data, Error>`.
   - *Rule*: **Zero panics, zero unwraps (`#![deny(clippy::unwrap_used)]`), zero `mut` by default**. 
   - Use persistent state structures (via `rpds` or `im` crates) instead of mutation. Use iterator pipelines (`itertools`) and suffix pipelines (`tap`) instead of imperative `for`/`while` loops where feasible.
   - *Rule*: Never put SQL queries, `reqwest` calls, or DOM manipulation inside `core/`.

3. **Actions (The Imperative Shell)**
   - Located in `diagram_tool/src/store_sqlx.rs`, `cli.rs`, or UI event handlers.
   - This is where side-effects, async runtimes (`tokio`), and I/O live.
   - We extract data (Action), pipe it into our pure core logic (Calc), and save the result (Action).

---

## 🎨 2. Dioxus 0.7 Frontend Constraints

Seshat pushes the DOM to its absolute limits to handle 3,000+ nodes smoothly at an 8ms frame budget (120 FPS).

### State Management & Reactivity
- **Signals over State**: Dioxus 0.7 relies heavily on `Copy` signals. Use `use_signal` for atomic values and `use_store` with `#[derive(Store)]` for nested collections. Never use the deprecated `use_state`.
- **Prop Passing**: Use `ReadSignal<T>` for component props that receive reactive values. It accepts `Signal`, `Memo`, `Resource`, or primitives with auto-conversion.
- **Context Injection**: Pass application-level state down via `use_context::<Signal<DiagramDocument>>()`. Mutate it using `doc_signal.with_mut(|doc| { ... })`.

### Performance Optimizations
- **Raw Event Listeners**: For extremely intense drag/pan/zoom interactions, we intentionally bypass standard Dioxus framework overhead. We mount vanilla JS listeners via `document::eval` and dispatch refined JSON events back to Dioxus via channels/messages (see `diagram_tool/src/ui/canvas.rs`).
- **DOM Stability**: Avoid unmounting/remounting large DOM subtrees. Keep the DOM stable and conditionally update CSS classes or transform positions using inline styles.
- **Tailwind**: Utilize Dioxus 0.7's automatic Tailwind integration (`asset!("/assets/tailwind.css")`).

### 🛑 WASM Build Constraints
- **CRITICAL**: The frontend compiles to `wasm32-unknown-unknown`. 
- You MUST NEVER include `tokio`, `mio`, `sqlx`, or `reqwest` (with default TLS features) in the UI code. 
- ALWAYS isolate backend/database dependencies behind `#[cfg(not(target_arch = "wasm32"))]`.
- The Dioxus `fullstack` feature MUST NOT be active when building purely for the web client.

---

## 🧪 3. Testing Strategy & Contract Protection

We employ an aggressive, multi-tiered testing strategy inspired by Dave Farley and Martin Fowler.

1. **Unit Tests (Pure Core)**: Since `core/` is entirely pure, we exhaustively test it. We use `proptest` for property-based fuzzing (e.g., verifying that a translation function is mathematically commutative).
2. **Acceptance & E2E**: Driven by Playwright (`npx playwright test`) to run headless browser simulations against the Dioxus app.
3. **Adversarial / Chaos Testing**: Ensuring that human and AI concurrent edits merge successfully and deterministically under heavy simulated load using our CRDT event-sourcing harness.

### 🛡️ Protected Contract Tests
Certain files are strictly designated as **CONTRACT TESTS**. They define the fundamental capabilities of the application. AI agents and human developers are **forbidden** from deleting, replacing, or "cleaning up" these tests without explicit permission.
Always verify rules in `.beads/TEST_PROTECTION.md`. Protected paths include:
- `diagram_tool/src/models/io_tests.rs` (IO-001 to IO-015)
- `diagram_tool/src/test_infrastructure_tests.rs`
- `diagram_tool/src/geometry/**/*.rs` (GEO-001 to GEO-030)

---

## 📝 4. Issue Tracking & The `bd` (Beads) Doctrine

Seshat utilizes **bd (beads)** for 100% of issue and task tracking. 
**Rule**: Do NOT use markdown TODOs, GitHub Issues, or external task managers.

### The Bead Lifecycle
1. **Check for ready work**: Run `bd ready --json` to find unblocked work items.
2. **Claim your work**: Lock the issue atomically using `bd update <id> --claim --json`.
3. **Discovering Scope**: If you find an unexpected bug or necessary refactor while working, create a linked issue:
   `bd create "Found bug" --description="..." -p 1 --deps discovered-from:<parent-id> --json`
4. **Complete Work**: Close it only when the code compiles and passes CI: `bd close <id> --reason "Completed" --json`.

Because beads are synced natively via Dolt, every write auto-commits to the Dolt history.

---

## 🛫 5. "Landing the Plane" (Full Moon Landing)

Work is never just "done locally." Before concluding a session or handing off an AI task, you must execute a strict **Full Moon Landing** protocol. 
If the Moon CI pipeline fails, or `git push` fails, the task is **not done**.

**The Mandatory Sequence:**
1. **File Remaining Work**: Use `bd create` for anything left incomplete.
2. **Run Quality Gates**: Execute `moon run :ci-source`. This triggers rustfmt, clippy (with strict `-D warnings`), and all unit tests. **It must pass.**
3. **Update Status**: Close or suspend your current `bd` issues.
4. **Sync and Push (CRITICAL)**:
   ```bash
   jj pull --rebase
   bd sync
   jj push
   jj status  # Must show "up to date with origin"
   ```
5. **Clean up**: Clear any local stashes or orphan branches. 
6. Never say "ready to push when you are" to a human. The AI MUST perform the push.

---

## 🛠️ 6. Core Tooling Commands

- **Local Dev Server (Hot Reload)**: 
  ```bash
  cd diagram_tool && dx serve --port 3333 --open false
  ```
- **Production Build**: 
  ```bash
  dx bundle
  ```
- **Rigorous CI / Validation**: 
  ```bash
  moon run :ci-source
  ```
- **End-to-End Tests**:
  ```bash
  npx playwright test
  ```