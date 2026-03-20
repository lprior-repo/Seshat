# Architecture Plan: Bead seshat-4eth (Phase 3 Pure Reducers)

## 1. Analysis of `handlers.rs`

The current UI handlers in `diagram_tool/src/ui/canvas/node_layer/handlers.rs` and `diagram_models/src/selection/handlers.rs` tightly couple Dioxus DOM events with domain business logic.

**Current Side Effects in `handle_mousedown` / `handle_mouseup`:**
*   **State Mutations:** Modifies `doc_signal` (DiagramDocument), `interaction_mode` (InteractionMode), `tool_signal` (ToolMode), and `space_pan_active`.
*   **History Mutations:** Pushes clones of the document into `history_signal`.
*   **External Effects:** Dispatches async DB updates via `db_tx`, shows warnings via `toast.show()`, and calls `flush_pending_pointer_update`.
*   **DOM Coupling:** Directly reads `evt.data.trigger_button()`, `evt.data.coordinates().client()`, and `multi_touch_active`.

To meet the requirement of "execute all business logic in pure reducer functions", the UI layer must be stripped of decision-making. The Dioxus handler's *only* job will be mapping DOM properties to a typed `CanvasEvent` and applying the result of a pure `apply_event` reducer.

## 2. Exact `CanvasEvent` Enum Variants Needed

The existing `CanvasEvent` (e.g., `MouseDownTarget`) is insufficient because it lacks target identity (`NodeId`), client coordinates (required for panning / anchor calculations), and modifier keys. 

To support the pure reducer, the `CanvasEvent` variants must be expanded/unified to capture exact interaction context:

```rust
use diagram_models::document::NodeId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanvasTarget {
    Node(NodeId),
    Background,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PointerButton {
    Primary,
    Secondary, // Right click
    Auxiliary, // Middle click
}

#[derive(Debug, Clone, PartialEq)]
pub enum CanvasEvent {
    PointerDown {
        target: CanvasTarget,
        canvas_pos: CanvasPoint,
        client_pos: CanvasPoint, // Needed for DraggingSelection / Panning anchor
        button: PointerButton,
        space_pressed: bool,
        mode: SelectionMode,
    },
    PointerMove {
        canvas_pos: CanvasPoint,
        client_pos: CanvasPoint,
        space_pressed: bool,
    },
    PointerUp {
        target: CanvasTarget,
        canvas_pos: CanvasPoint,
        client_pos: CanvasPoint,
    },
    // We retain or map Touch/Drag movements logically as Pointer variants 
    // to unify mouse and touch workflows in the reducer.
}
```

## 3. Pure Reducer Architecture

Since the handlers currently trigger side effects (toasts, history pushes), the pure reducer must return both the new state and a list of declarative effects to be executed by the UI shell.

```rust
pub struct CanvasState {
    pub document: DiagramDocument,
    pub interaction_mode: InteractionMode,
    pub tool_mode: ToolMode,
    pub space_pan_active: bool,
}

pub enum CanvasEffect {
    PushHistory(DiagramDocument),
    ShowToastWarning(String),
    FlushPointerUpdate,
}

/// The core pure function that replaces `handle_mousedown` and `handle_mouseup`
pub fn apply_event(
    state: CanvasState,
    event: CanvasEvent,
) -> Result<(CanvasState, Vec<CanvasEffect>), CanvasError> {
    // ... Pure business logic (DAG circular checks, selection toggling, etc)
}
```

## 4. Implementation Strategy (Phase 3)

1.  **Domain Types:** Update `CanvasEvent`, `RawEvent`, and parsing logic in `canvas_domain/src/canvas_event.rs` and `types.rs`.
2.  **Pure Reducer:** Create `apply_event` taking `CanvasState` and `CanvasEvent`. Migrate the DAG checking (`edge_preserves_dag`), selection toggling (`toggle_selection`), and interaction mode setting (`InteractionMode::DrawingEdge`, `DraggingSelection`) from `handlers.rs` into this pure function.
3.  **Refactor UI Shell:** Update `diagram_tool/src/ui/canvas/node_layer/handlers.rs`. It will now simply:
    *   Construct `CanvasEvent`.
    *   Read current state into `CanvasState`.
    *   Call `apply_event(state, event)`.
    *   Write the updated state back to Dioxus `Signal`s.
    *   Iterate over `Vec<CanvasEffect>` and execute side effects (e.g. `toast.show`).
4.  **Test Verification:** Ensure no unwrap/expect calls are introduced, maintaining the functional Rust "zero panics" constraints, and fixing combinatorial tests in `interaction_combinatorial_tests.rs`.