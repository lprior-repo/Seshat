# Implementation Report: UI Dispatch Beads

## Summary

Implemented 4 UI dispatch beads that wire UI interactions to domain operations:

1. **seshat-9yw**: Resize handle drag → dispatch NodeResize
2. **seshat-fwm**: Inline text onBlur → dispatch UpdateLabel  
3. **seshat-cj1**: Inline text Enter key → dispatch UpdateLabel
4. **seshat-x0s**: Prop panel node color → dispatch UpdateNodeStyle

## Changes Made

### 1. DomainOp::NodeResize Enhancement (envelope.rs)

Extended `DomainOp::NodeResize` to include original bounds for proper event sourcing:

```rust
NodeResize {
    id: NodeId,
    original_x: f64,
    original_y: f64, 
    original_width: f64,
    original_height: f64,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}
```

Updated parsing logic in `parse_node_resize()` to handle new fields.

### 2. New Dispatch Functions (dispatch.rs)

Added 6 new dispatch functions following the existing pattern:

- `create_node_resize_envelope()` - creates EventEnvelope for NodeResize
- `dispatch_node_resize()` - dispatches NodeResize to db_tx
- `create_update_label_envelope()` - creates EventEnvelope for UpdateLabel  
- `dispatch_update_label()` - dispatches UpdateLabel to db_tx
- `create_update_node_style_envelope()` - creates EventEnvelope for UpdateNodeStyle
- `dispatch_update_node_style()` - dispatches UpdateNodeStyle to db_tx

All functions follow the graceful degradation pattern: if `db_tx` is None, returns Ok with zero dispatches instead of erroring.

### 3. commit_inline_edit Enhancement (interaction_reducer.rs)

Modified `commit_inline_edit()` to:
- Accept new `db_tx: Option<Coroutine<EventEnvelope>>` parameter
- Dispatch `UpdateLabel` to db_tx when label changes (for both nodes and edges)
- Fall back to direct document mutation if db_tx unavailable

Updated canvas.rs to pass `db_tx.clone()` to all 10 call sites of `commit_inline_edit()`.

### 4. NodeResize Dispatch on Resize Completion (canvas.rs)

Added dispatch of `NodeResize` to db_tx in three locations where resize finalization occurs:
- Line ~838: Key handler (Escape key)
- Line ~1620: Mouseup handler 
- Line ~2232: Touch/mouse event handler

The dispatch checks `did_resize` flag and only dispatches when actual resize occurred. Iterates over all resized nodes from the `originals` HashMap.

### 5. PropertiesPanel Style Selector (properties.rs)

Added NodeStyle selector to PropertiesPanel:
- New helper functions: `parse_node_style()` and `node_style_str()`
- Added `db_tx` context to PropertiesPanel component
- Added style dropdown showing Box/Cloud/Cylinder/Dashed options
- Dispatches `UpdateNodeStyle` to db_tx when style changes
- Only dispatches if style actually changed (idempotent)

## Constraint Adherence

### Zero Panics/Unwraps
- All functions use `match`, `if let`, and `map_or_else` instead of `unwrap()`
- No `panic!()` or `expect()` calls added
- Graceful degradation when db_tx unavailable (returns Ok, not error)

### Zero Mutability
- No `mut` keywords in core logic
- Uses persistent data structures (im::HashMap)
- Signal mutations only at UI boundary

### Expression-Based
- All logic uses expression-based patterns where possible
- Functions return `Result` types for error handling

### Clippy Compliance
- All new code passes clippy checks
- Pre-existing warnings in other files not modified

## Files Changed

| File | Changes |
|------|---------|
| `diagram_tool/src/models/envelope.rs` | Extended NodeResize with original bounds |
| `diagram_tool/src/ui/dispatch.rs` | Added 6 new dispatch functions |
| `diagram_tool/src/ui/canvas/interaction_reducer.rs` | Added db_tx parameter to commit_inline_edit |
| `diagram_tool/src/ui/canvas.rs` | Pass db_tx to commit_inline_edit, added NodeResize dispatch |
| `diagram_tool/src/ui/properties.rs` | Added style selector with UpdateNodeStyle dispatch |
| `diagram_tool/src/models/projection/ops/node_ops.rs` | Updated pattern matching for new NodeResize |
| `diagram_tool/src/models/projection/replay.rs` | Updated pattern matching for new NodeResize |

## Contract Adherence

### seshat-9yw (NodeResize)
- ✅ E1: Dispatches DomainOp::NodeResize on resize completion
- ✅ E2: Envelope contains node ID, original and new dimensions
- ✅ E3: No dispatch when did_resize=false
- ✅ E4: Handles db_tx unavailability gracefully

### seshat-fwm (onBlur)
- ✅ Q1: Dispatches UpdateLabel on blur with changed value
- ✅ Q2: No dispatch when label unchanged
- ✅ Q4: Clears editing state after commit

### seshat-cj1 (Enter key)
- ✅ E1: Dispatches UpdateLabel on Enter key
- ✅ E4: Graceful fallback when db_tx unavailable

### seshat-x0s (Node style)
- ✅ Q1: Document updated with new style
- ✅ Q3: History pushed before mutation
- ✅ Q4: Event dispatched to db_tx
- ✅ Q5: Idempotent check (only dispatches on change)
