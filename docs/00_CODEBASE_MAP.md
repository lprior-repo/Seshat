# Codebase Map

This document serves as a definitive guide to the `diagram_tool` directory structure, helping AI agents and developers quickly locate where different concerns of the application live.

## High-Level Structure

The codebase is split into specific layers enforcing our Data -> Calc -> Actions architectural pattern.

### `diagram_tool/src/models/` (Data)
Contains pure, immutable domain models defining the state of the diagram. 
- **Key files**: `document.rs` (Node, Edge, DocumentData), `schema_defs.rs` (SQLite schema definitions), `conflict.rs` (Conflict resolution models).
- **Rule**: No side-effects, no persistence, no UI logic.

### `diagram_tool/src/core/` and `diagram_tool/src/mutation/` (Calc)
Contains pure calculations and operations that transition the Data from one state to another.
- **Key files**: `core/routing.rs` (Edge routing math), `core/z_order.rs`, `mutation/ops.rs`.
- **Rule**: Functions here should take input data, perform calculations, and return a `Result<NewData, Error>`. No database writes or DOM updates occur here.

### `diagram_tool/src/ui/` (Frontend / Dioxus)
The presentation layer built with Dioxus 0.7.
- **Key directories**: `canvas/` (Canvas interaction, rendering), `toolbar/`, `panels/`.
- **Key files**: `canvas.rs` (Main canvas component, input handling), `commands.rs` (UI command abstractions).
- **Rule**: Dioxus UI pushes the DOM to its limits. Avoid WASM-incompatible operations.

### `diagram_tool/src/` (Root / Actions / Integration)
Contains persistence, CLI, and integration logic.
- **Key files**: `store_sqlx.rs` (SQLite operations), `cli.rs` (AI CLI contract handling), `backend.rs`, `app.rs` (Main app orchestration).
- **Rule**: This is where side effects happen (Action). State changes are committed to SQLite or sent over the network.

### `diagram_tool/src/geometry/` and `diagram_tool/src/layout/`
Math, geometry operations, snapping, and `petgraph` integration for DAG layout.

### `diagram_tool/src/perf/`
Performance metrics, FPS tracking, and regression testing tools.
