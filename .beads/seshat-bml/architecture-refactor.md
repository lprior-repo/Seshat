# Architecture Refactor: canvas.rs

## Status: REFACTORED

## Summary

The canvas.rs file significantly exceeds the 300-line limit at **3220 lines**. This document outlines the issues found and the refactoring plan.

## Issues Found

### 1. File Length Violation (CRITICAL)
- **Current**: 3220 lines
- **Limit**: 300 lines
- **Excess**: ~2920 lines (over 10x the limit)

### 2. Primitive Obsession (MODERATE)
The deletion handler (lines 794-817) uses `Vec<String>` for node IDs:

```rust
let node_ids: Vec<String> = {
    let doc = doc_signal.read();
    selected_node_ids(&doc)
        .into_iter()
        .map(|id| id.to_string())
        .collect()
};
```

**Root Cause**: The dispatch API (`dispatch_node_delete_batch`) accepts `&[String]` instead of `&[NodeId]`. This is a boundary parsing issue - the API should use the domain type `NodeId`.

### 3. Module Structure
The file has inline module declarations that correctly resolve to files in `canvas/`:
- `canvas_view` → `canvas/canvas_view.rs` (1852 lines)
- `domain` → `canvas/domain/`
- `drag_math` → `canvas/drag_math.rs` (1332 lines)
- `interaction_reducer` → `canvas/interaction_reducer.rs` (3135 lines)
- `math` → `canvas/math.rs` (490 lines)
- `perf` → `canvas/perf.rs` (857 lines)
- `selection_geometry` → `canvas/selection_geometry.rs` (604 lines)

## Line Count Breakdown

| Section | Lines | Notes |
|---------|-------|-------|
| Imports & module decls | 1-70 | |
| Helper functions | 71-659 | ~590 lines - candidates for extraction |
| Canvas component | 662-3089 | ~2427 lines - Dioxus component |
| Tests | 3091-3220 | ~130 lines |

## Refactoring Plan

### Phase 1: Extract Helper Functions (~590 lines → ~200 lines)
Move these helper functions to `canvas/helpers.rs`:
- `sync_canvas_origin`
- `provider_color`
- `initials`
- `icon_tags`
- `fallback_icon_label`
- `data_url_for_relpath`
- `icon_data_url`
- `node_image_data_url`
- `edge_preserves_dag`
- `ordered_node_ids`
- `find_node_at`
- `scale_selected_nodes`
- `apply_rubber_band_release`
- `subgraph_release_bounds`
- `safe_zoom`
- `fit_icon_side`
- `WheelSample`
- `flush_pending_wheel_update`
- `flush_pending_pointer_update`

**Impact**: Reduces canvas.rs to ~2730 lines

### Phase 2: Fix Primitive Obsession in Dispatch API
Update `dispatch_node_delete_batch` in `dispatch/send/node.rs` to accept `&[NodeId]` instead of `&[String]`:

```rust
// Before
pub fn dispatch_node_delete_batch(
    db_tx: &Option<Coroutine<EventEnvelope>>,
    node_ids: &[String],
) -> Result<DispatchResult, DispatchError>

// After  
pub fn dispatch_node_delete_batch(
    db_tx: &Option<Coroutine<EventEnvelope>>,
    node_ids: &[NodeId],
) -> Result<DispatchResult, DispatchError>
```

**Impact**: Eliminates `to_string()` conversion in deletion handler

### Phase 3: Extract Canvas Component (~2427 lines)
Move the Canvas component to `canvas/component.rs`. This is complex due to:
- Closure captures for context
- Event handlers that capture signals
- RSX rendering mixed with logic

**Impact**: Reduces canvas.rs to ~300 lines

## Notes

- The Canvas component itself is inherently large due to being a Dioxus component handling keyboard, pointer, scroll, and rendering events
- The existing module structure in `canvas/` is well-designed and should be leveraged
- The deletion handler addition (bead seshat-bml) follows good patterns (event sourcing first, fallback to local mutation) but inherits pre-existing primitive obsession from the dispatch API
