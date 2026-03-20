# Architecture Specification: Pure Dioxus 0.7 & Scott Wlaschin DDD

## 1. EARS (Requirements & Triggers)
- **When** the application initializes, the system shall instantiate a single centralized `#[derive(Store)] AppState` containing the entire application state.
- **When** a user interacts with the canvas (e.g., mouse down, key press), the UI components shall parse the raw DOM event into a typed `DomainEvent` (e.g., `CanvasEvent::NodeClicked`).
- **When** a `DomainEvent` is dispatched, the system shall invoke a pure reducer `fn apply_event(state: &AppState, event: DomainEvent) -> Result<Vec<Mutation>, DomainError>`.
- **While** in the UI rendering phase, components shall ONLY consume `ReadSignal<T>` or `Store<T>` and shall NOT perform document mutations or complex business logic inline.
- **If** a pure reducer determines an action requires I/O (e.g., exporting a PNG, saving to DB), the system shall emit an `Effect` to a `use_coroutine` at the shell boundary, strictly separating I/O from the core.

## 2. KIRK Contracts & Domain Modeling (DDD)
### Preconditions & Invariants
- **Invariant:** `SnapResult` cannot contain positional data if snapping is disabled. (Enforced by `enum SnapResult { Snapped { pos: Point, target: NodeId }, Unsnapped }`).
- **Invariant:** The Core Domain model shall not contain any UI-specific types (e.g., `web_sys::MouseEvent`).
- **Invariant:** All CSS must be specified using Tailwind classes. `style` strings are banned for static visual properties.
- **Precondition:** `document::eval` shall not be used for DOM manipulation if a native Rust/Dioxus alternative (`use_resource`, `gloo`, `web_sys`) exists.

### Type-Level Enforcement (Make Illegal States Unrepresentable)
- **Boolean Soup Removal:** Functions taking multiple boolean flags (e.g., `ctrl_pressed`, `shift_pressed`, `snap_to_grid`) will be refactored to specific intention Enums and bitflags (e.g., `enum GridSnap { Disabled, Enabled(GridSize) }`).
- **Zero Panics:** No `unwrap()` or `expect()` in core domain logic. All fallible operations must return `Result<T, DomainError>`.
- **Parse, Don't Validate:** Raw inputs (mouse coordinates) are parsed into validated domain commands before passing into core functions.

## 3. Inversion & Exhaustive Error Taxonomy
- `DomainError::CircularDependency`: Emitted when an edge creation would violate the DAG constraints.
- `DomainError::NodeNotFound(NodeId)`: Emitted when an operation targets a non-existent node.
- `DomainError::InvalidStateTransition`: Emitted when an event is dispatched in an incompatible state (e.g., `CommitEdge` while `State::Idle`).
- `ExportError::RenderFailed`: Emitted when SVG -> PNG rasterization fails.
- `PersistenceError::StorageUnavailable`: Emitted when `localStorage` cannot be accessed natively.

## 4. Execution Phases

### Phase 1: The Store Consolidation (Dioxus 0.7)
- Remove the 15 scattered `Signal::new()` calls from `app/mod.rs`.
- Define a central `#[derive(Store)] AppState`.
- Update all 63 components currently taking `mut doc_signal: Signal<DiagramDocument>` to use `ReadSignal<T>` or read directly from the Store.

### Phase 2: The Tailwind Sweep & JS Purge
- Replace 188+ inline `style="..."` strings with declarative Tailwind `class="..."` attributes.
- Replace 24 `document::eval` calls (for timeouts and local storage) with Dioxus native `use_coroutine`, `async_std::task::sleep`, or standard `web_sys::window().local_storage()`.

### Phase 3: Pure Handlers & UI Decoupling
- Extract all business logic (UUID generation, cycle detection, history management) from `ui/canvas/node_layer/handlers.rs`.
- Introduce explicit `CanvasEvent` commands and pure `reduce()` functions.

### Phase 4: Strict DDD & Zero Panics
- Refactor `SnapResult` and `core::keyboard` to eliminate boolean flags in favor of typed enums.
- Purge the 13 remaining `unwrap()` calls in production code by mapping them to explicit `thiserror` variants.
