# whiteboard_kit — Extracted Drag, Drop, Whiteboard & JSON Export Code

This folder is a self-contained extraction of every file related to:

- **Drag and drop** of canvas nodes
- **Whiteboard / canvas** interaction (pan, zoom, marquee selection, connection drawing)
- **Rearrange** (auto-layout via DAG toposort, fit-view, snap-to-grid)
- **JSON export** (download workflow as `.json` file via browser Blob API)
- **Workflow execution** (topological run engine, condition branching, per-step history)
- **Validation** (structural checks → ValidationIssue list)
- **UI panels** (sidebar, inspector, config panel, execution history, run status bar)
- **Workflow nodes** (HTTP trigger, schedule, service call, send message, delay, router, run code, wait for webhook/signal, memory ops)

Copy the sub-folders you need into your new app and adapt the crate name
(`oya_frontend`) to your own.

---

## Complete File Index

```
whiteboard_kit/
│
├── WHAT_IS_THIS.md             ← you are here
├── AGENTS.md                   Agent / quality-gate instructions (copy of root)
├── CLAUDE.md                   Claude Code project instructions (copy of root)
├── contract-spec.md            Design-by-contract spec for the whole app
├── martin-fowler-tests.md      Test taxonomy and naming conventions
├── dioxus_tailwind_ai_guide_review.md  Dioxus 0.7 + Tailwind AI guide
│
├── Cargo.toml                  Full workspace dependencies
├── Dioxus.toml                 Dioxus web platform config
├── moon.yml                    Moon task definitions for this crate
│
├── .moon/
│   ├── toolchain.yml           Moon toolchain config (Rust, Node versions)
│   ├── workspace.yml           Moon workspace layout
│   └── tasks.yml               Shared Moon task definitions
│
├── assets/
│   └── tailwind.css            Tailwind CSS entry point (@import "tailwindcss")
│
├── style.css                   Full custom CSS (variables, node cards, sidebar,
│                               minimap, edge animations, dark mode tokens)
│
├── docs/
│   ├── 01_ERROR_HANDLING.md    Error handling conventions (no unwrap/expect/panic)
│   ├── 02_MOON_BUILD.md        Moon build system guide
│   ├── 03_WORKFLOW.md          Development workflow and Git conventions
│   ├── 05_RUST_STANDARDS.md    Rust coding standards for this project
│   ├── 08_BEADS.md             Bead-driven planning system docs
│   ├── 09_JUJUTSU.md           Jujutsu VCS guide
│   ├── 10_RESTATE_SDK.md       Restate SDK integration docs
│   ├── 11_FLOW_EXTENDER.md     Flow extender module docs
│   └── plans/
│       ├── 2026-02-20-refactor-hooks-canvas.md   Detailed hook refactor plan with full code
│       └── comfyui-minimal-canvas-redesign.md    Product design doc (ComfyUI-inspired canvas)
│
├── errors.rs                   Shared error types (AppError, etc.)
│
├── graph/                      Core data model — no UI, no Dioxus
│   ├── mod.rs                  Node, Connection, Viewport, Workflow, NodeId types + serde
│   ├── calc.rs                 Pure math: update_node_position, zoom, pan, fit_view, snap
│   ├── core.rs                 Workflow mutation: add_node, remove_node, update_node_position
│   ├── connectivity.rs         add_connection with cycle / duplicate / self-loop guards
│   ├── execution.rs            Workflow run engine (topological execution, skip logic)
│   ├── execution_record.rs     Per-step execution history records
│   ├── execution_state.rs      ExecutionState enum (Idle/Running/Succeeded/Failed/…)
│   ├── expressions.rs          Expression evaluation for condition nodes
│   ├── layout.rs               DAG auto-layout (petgraph toposort + barycenter sweep)
│   ├── metadata.rs             Node type → (category, icon, description) lookup
│   ├── restate_types.rs        Restate-specific type definitions
│   ├── validation.rs           Workflow validation rules → ValidationIssue list
│   ├── view.rs                 Viewport zoom/pan helpers (fit_view, zoom_at_point)
│   └── workflow_node.rs        Per-node-type execution logic
│
├── hooks/                      Dioxus 0.7 reactive hooks (use_signal / use_memo)
│   ├── mod.rs                  Re-exports
│   ├── use_canvas_interaction.rs   InteractionMode state machine (Idle/Panning/Dragging/Connecting/Marquee)
│   ├── use_frozen_mode.rs      Read-only replay overlay for historical run records
│   ├── use_selection.rs        Multi-node selection state (select, toggle, marquee result)
│   ├── use_sidebar.rs          Sidebar open/close + search state
│   ├── use_ui_panels.rs        Inspector / config panel visibility toggles
│   └── use_workflow_state.rs   Workflow + undo/redo stack + memoised derived views
│
├── ui/                         Dioxus 0.7 components
│   ├── mod.rs                  UI module re-exports
│   ├── app_bootstrap.rs        App entry point / bootstrap component
│   ├── app_io.rs               download_workflow_json (WASM Blob + anchor click), canvas helpers
│   ├── canvas_context_menu.rs  Right-click context menu on the canvas
│   ├── command_palette.rs      Command palette (Cmd+K style)
│   ├── edges.rs                FlowEdges — SVG bezier edges with bend handles + animated running state
│   ├── editor_interactions.rs  Pure helpers: normalize_rect, node_intersects_rect, snap_handle
│   ├── execution_history_panel.rs  Panel showing per-step execution history
│   ├── execution_plan_panel.rs     Panel showing the planned execution order
│   ├── expression_input.rs     Expression / template input component
│   ├── inline_config_panel.rs  Inline (canvas-embedded) config panel
│   ├── inspector_panel.rs      Node inspector side panel
│   ├── interaction_guards.rs   Prevents duplicate interactions during animation frames
│   ├── minimap.rs              FlowMinimap — SVG overview of the entire canvas
│   ├── node.rs                 FlowNodeComponent — draggable card with handles, status badge, output preview
│   ├── parallel_group_overlay.rs  Visual overlay for parallel node groups
│   ├── prototype_palette.rs    PrototypePalette — modal node picker + skeleton YAML generator
│   ├── run_status_bar.rs       Run status bar (progress, errors, stop button)
│   ├── selected_node_panel.rs  Panel for currently-selected node (uses flow_extender)
│   ├── toolbar.rs              FlowToolbar — zoom, fit, layout, undo/redo, run, save buttons
│   ├── validation_panel.rs     Validation issues panel
│   │
│   ├── config_panel/
│   │   ├── mod.rs              Config panel entry + dispatch
│   │   ├── common.rs           Shared config panel form primitives
│   │   ├── config_sections.rs  Per-section config forms
│   │   └── execution.rs        Execution config (WASM: wasm_bindgen, web_sys, gloo_timers)
│   │
│   ├── icons/
│   │   ├── mod.rs              Icon system re-exports
│   │   ├── registry.rs         Icon lookup by name
│   │   ├── set_a.rs            Icon set A (SVG inline components)
│   │   ├── set_b.rs            Icon set B
│   │   └── set_c.rs            Icon set C
│   │
│   ├── sidebar/
│   │   ├── mod.rs              Sidebar module entry
│   │   ├── model.rs            Sidebar data model (categories, node templates)
│   │   ├── presentation.rs     Sidebar UI component
│   │   └── tests.rs            Sidebar unit tests
│   │
│   └── workflow_nodes/         Per-node-type card + form components
│       ├── mod.rs              Node type dispatch
│       ├── schema.rs           Shared node schema/form types
│       ├── http_trigger.rs     HTTP Trigger node card + config form
│       ├── schedule_trigger.rs Schedule Trigger node
│       ├── service_call.rs     Service Call node
│       ├── delayed_message.rs  Delayed Message node
│       ├── save_to_memory.rs   Save to Memory node
│       ├── load_from_memory.rs Load from Memory node
│       ├── delay.rs            Delay node
│       ├── router.rs           Router (condition branching) node
│       ├── wait_for_webhook.rs Wait for Webhook node (also defines WaitForSignal*)
│       ├── wait_for_signal.rs  Wait for Signal node
│       ├── run_code.rs         Run Code node
│       └── send_message/
│           └── mod.rs          Send Message node
│
└── tests/                      All tests — verbatim copies
    ├── drag_unit_test.rs               Unit: update_node_position (NaN, Inf, clamp, snap)
    ├── node_drag_regression.rs         Unit: node stays visible during mousedown + drag
    ├── graph_layout_regressions.rs     Unit: layout idempotency, positive bounds, fit-view stability
    ├── graph_regressions.rs            Unit: workflow run, condition branching, cycle rejection, history cap
    ├── e2e_acceptance.rs               Rust E2E acceptance tests
    ├── ui_test.rs                      Component-level tests
    ├── flow-editor.spec.ts             Playwright E2E: add node, drag, connect, delete, undo/redo
    ├── flow-editor-advanced.spec.ts    Playwright E2E: multi-select, marquee, zoom, JSON export
    ├── flow-editor-adversarial.spec.ts Playwright E2E: chaos / adversarial edge cases
    ├── flow-helpers.ts                 Shared Playwright helpers (waitForNode, dragNode, etc.)
    └── component.spec.js               JS component smoke tests
```

---

## Key Concepts to Port

### 1. Drag and Drop (`use_canvas_interaction` + `use_selection`)

The interaction is a **state machine** (`InteractionMode`):

```
Idle → Panning         (space+mousedown on canvas)
Idle → Dragging{ids}   (mousedown on node, threshold exceeded)
Idle → Connecting{…}   (mousedown on handle)
Idle → Marquee{start}  (mousedown on empty canvas)
any  → Idle            (mouseup / escape)
```

Node position update on every `mousemove` during `Dragging`:
```rust
// calc.rs
let new_x = ((current_x + dx) / 10.0).round() * 10.0;  // snap to 10px grid
```

Mouse delta is computed from the raw `mousemove` event against a stored
`drag_anchor` position recorded at `mousedown`.

### 2. Auto-Layout (`graph/layout.rs`)

Uses `petgraph` for DAG toposort + barycenter crossing minimisation.
Call `workflow.apply_layout()` — positions are reassigned in-place.

### 3. JSON Export (`ui/app_io.rs`)

WASM-only. Calls `serde_json::to_string_pretty`, creates a `Blob`, constructs
a temporary `<a>` element, clicks it, then revokes the object URL. The
`Workflow` struct derives `Serialize`/`Deserialize` so round-trips are free.

### 4. Undo/Redo (`hooks/use_workflow_state.rs`)

Simple snapshot stack capped at 60 entries. Before any mutation call
`save_undo_point()` which clones the current `Workflow` onto `undo_stack`
and clears `redo_stack`.

---

## Known Compile Considerations

- **`selected_node_panel.rs`** imports from `oya_frontend::flow_extender` — an
  app-specific module not included here. You'll need to stub or replace this
  import for standalone compilation.

- **`config_panel/execution.rs`** uses `wasm_bindgen`, `web_sys`, `js_sys`,
  `gloo_timers`, `wasm_bindgen_futures` — these only compile for
  `target_arch = "wasm32"`. Use `#[cfg(target_arch = "wasm32")]` guards if
  building for non-WASM targets.

- **`wait_for_webhook.rs`** and **`wait_for_signal.rs`** both define
  `WaitForSignalForm` and `WaitForSignalNodeCard` — this is intentional
  duplication from the source; verify which one your app should use.

---

## Dependencies You'll Need

```toml
[dependencies]
dioxus = { version = "0.7", features = ["web"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4", "serde"] }
petgraph = "0.6"
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1"
itertools = "0.12"

# WASM targets only
[target.'cfg(target_arch = "wasm32")'.dependencies]
web-sys = { version = "0.3", features = ["Blob", "HtmlAnchorElement", "Url", "Window", "Document"] }
js-sys = "0.3"
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
gloo-timers = { version = "0.3", features = ["futures"] }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

---

## Build System Notes

The original project uses **Moon** as the build system. The canonical CI command is:

```sh
moon run :ci --force
```

Individual tasks:
```sh
moon run :check      # cargo check
moon run :fmt        # cargo fmt
moon run :clippy     # cargo clippy
moon run :test       # cargo test
moon run :build-web  # dx build --platform web
moon run :serve      # dx serve (hot reload)
```

Do **not** call `cargo`, `dx`, or `npm` directly — always go through Moon.
