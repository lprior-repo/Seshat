# Implementation Plan: Phase 1 Dioxus Store (`AppState`)

## 1. Context Analysis: The 15 Global State Signals
In `diagram_tool/src/app/mod.rs`, there are exactly 15 distinct `Signal` instances provided globally via `use_context_provider(|| Signal::new(...))`. Providing primitive types directly as global contexts is an anti-pattern because it causes type collisions (e.g., any other `Signal<bool>` or `Signal<u64>` added later would overwrite the existing ones). 

The 15 signals mapped to their specific domain purposes are:

**Core & Document**
1. `Signal<DiagramDocument>` - Core document state
2. `Signal<History>` - Undo/Redo tracking
3. `Signal<Option<ClipboardData>>` - Copy/Paste buffer

**Editor State**
4. `Signal<ToolMode>` - Current active tool (Select, Node, Edge, etc.)
5. `Signal<EdgeStyle>` - Default edge style
6. `Signal<ArrowType>` - Default arrow type

**UI & Layout**
7. `Signal<Option<DraggedIconPayload>>` - Sidebar-to-Canvas drag state
8. `Signal<SidebarUiState>` - Mobile/Sidebar visibility
9. `Signal<ToolbarStats>` - Node/Edge counts displayed in the UI
10. `Signal<(f64, f64)>` - Canvas `viewport_size`

**Sync & Conflicts**
11. `Signal<Option<AiConflictState>>` - AI merge conflict state
12. `Signal<bool>` - Tracks if `conflict_toast_shown`
13. `Signal<HashSet<String>>` - Tracks `pending_ai_ops`

**Testing & System**
14. `Signal<u64>` - E2E `validate_trigger` sequence ID
15. `Signal<ToastQueue>` - Active toast notifications

---

## 2. Structural Change: `AppState` Struct

**File:** `diagram_tool/src/app/state.rs` (New File)

Create a unified `AppState` structure holding these 15 signals. Because `Signal` itself is `Copy`, `AppState` can comfortably be `Clone + Copy`, making it effortless to pass around just like native Dioxus hooks.

```rust
use crate::history::History;
use crate::ui::commands::ClipboardData;
use crate::ui::editor::ToolMode;
use crate::ui::mobile::SidebarUiState;
use crate::ui::toast::{AiConflictState, ToastQueue};
use crate::ui::toolbar::ToolbarStats;
use crate::app::types::DraggedIconPayload;
use diagram_models::document::{ArrowType, DiagramDocument, EdgeStyle};
use dioxus::prelude::*;
use std::collections::HashSet;

#[derive(Clone, Copy)]
pub struct AppState {
    pub document: Signal<DiagramDocument>,
    pub history: Signal<History>,
    pub clipboard: Signal<Option<ClipboardData>>,
    pub tool_mode: Signal<ToolMode>,
    pub edge_style: Signal<EdgeStyle>,
    pub arrow_type: Signal<ArrowType>,
    pub dragging_icon: Signal<Option<DraggedIconPayload>>,
    pub sidebar: Signal<SidebarUiState>,
    pub toolbar_stats: Signal<ToolbarStats>,
    pub viewport_size: Signal<(f64, f64)>,
    pub ai_conflict: Signal<Option<AiConflictState>>,
    pub conflict_toast_shown: Signal<bool>,
    pub pending_ai_ops: Signal<HashSet<String>>,
    pub validate_trigger: Signal<u64>,
    pub toasts: Signal<ToastQueue>,
}

impl AppState {
    pub fn provide() -> Self {
        use_context_provider(|| Self {
            document: Signal::new(DiagramDocument::default()),
            history: Signal::new(History::new()),
            clipboard: Signal::new(None),
            tool_mode: Signal::new(ToolMode::Select),
            edge_style: Signal::new(EdgeStyle::Solid),
            arrow_type: Signal::new(ArrowType::Default),
            dragging_icon: Signal::new(None),
            sidebar: Signal::new(SidebarUiState::default()),
            toolbar_stats: Signal::new(ToolbarStats::default()),
            viewport_size: Signal::new((0.0_f64, 0.0_f64)),
            ai_conflict: Signal::new(None),
            conflict_toast_shown: Signal::new(false),
            pending_ai_ops: Signal::new(HashSet::new()),
            validate_trigger: Signal::new(0_u64),
            toasts: Signal::new(ToastQueue::default()),
        })
    }
}
```

Don't forget to expose it in `diagram_tool/src/app/mod.rs`:
```rust
mod state;
pub use state::AppState;
```

---

## 3. Modifying the `App` Component

**File:** `diagram_tool/src/app/mod.rs`

Delete all 15 `use_context_provider(|| Signal::new(...))` lines. Replace them with:
```rust
let state = AppState::provide();
```

Inside `App`, update any direct `use_context` lookups:
```rust
// Replace:
let doc_signal = use_context::<Signal<DiagramDocument>>();
let mut toolbar_stats = use_context::<Signal<ToolbarStats>>();

// With:
let doc_signal = state.document;
let mut toolbar_stats = state.toolbar_stats;
```

---

## 4. System-Wide Refactoring (The Consumers)

Anywhere the codebase currently calls `use_context::<Signal<Type>>()`, it must be updated to use `use_context::<AppState>().field_name`. 

**Example Transformation:**
```rust
// Old:
let tool_signal = use_context::<Signal<ToolMode>>();

// New:
let app_state = use_context::<AppState>();
let tool_signal = app_state.tool_mode;
```

### Targeted Files to Refactor

1. **`diagram_tool/src/ui/canvas/state.rs`**
   - Update `use_canvas_state()` which heavily reads global context. Fetch `app_state` once and map out `document`, `dragging_icon`, `history`, `tool_mode`, `edge_style`, `arrow_type`, and `viewport_size`.

2. **`diagram_tool/src/hooks/e2e_reset.rs`**
   - Update `use_e2e_reset_hook()` which clears out `doc_signal`, `history_signal`, `tool_mode`, `edge_style`, `arrow_type`, `toast_queue`, `toolbar_stats`, `viewport_size`, and `validate_trigger`.

3. **`diagram_tool/src/ui/toolbar.rs`**
   - Updates needed for fetching doc, history, tools, toasts, stats, and viewport size.

4. **`diagram_tool/src/app/async_sync.rs`**
   - Refactor fetching `ToastQueue`, `AiConflictState`, `conflict_toast_shown` (the raw bool), and `pending_ai_ops` (the HashSet).

5. **`diagram_tool/src/hooks/keyboard.rs`**
   - Fix lookups for `doc_signal`, `history_signal`, and `clipboard_signal`.

6. **`diagram_tool/src/app/autosave_hooks.rs`**
   - Fix lookups for `ToolMode`, `EdgeStyle`, `ArrowType`.

7. **`diagram_tool/src/ui/properties.rs`**
   - Fix lookups for `DiagramDocument` and `History`.

8. **`diagram_tool/src/ui/sidebar/mod.rs`**
   - Fix lookups for `DraggedIconPayload` and `SidebarUiState`.

9. **`diagram_tool/src/ui/toast/render.rs` & `diagram_tool/src/ui/toast/mod.rs`**
   - Refactor where `ToastQueue` and `AiConflictState` are consumed. Note: `use_toast()` can fetch `AppState` internally to get the queue.

10. **`diagram_tool/src/ui/canvas/node_layer/node_element.rs`**
    - Refactor `edge_style_default` and `arrow_type_default` lookups.

---

## 5. Build and Test Validation

Once the refactoring is complete, execute the rigorous `moon` pipeline to guarantee strict `clippy-source` compliance and zero breakage:

```bash
moon run :ci-source
```
If any unmapped signals exist, the Rust compiler will immediately flag them since the global `Signal<T>` contexts will no longer be provided. This makes the migration safely deterministic.