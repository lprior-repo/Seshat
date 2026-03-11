# Implementation Report: seshat-c0j (NodeResize Projection)

## Summary
Fixed Black Hat defects for NodeResize projection:
1. ✅ **apply_node_resize exceeds 25 lines** - Refactored to 16 lines (body)
2. ✅ **Error type mismatch** - Now uses `ProjectionError` (not `ReplayError`)

## Changes Made

### 1. Added `ProjectionError` enum
**File**: `diagram_tool/src/models/projection/types.rs`
```rust
pub enum ProjectionError {
    NodeNotFound(String),
    InvalidDimensions(String),
    InvalidOperation(String),
}
```

### 2. Refactored `apply_node_resize` (≤25 lines)
**File**: `diagram_tool/src/models/projection/ops/node_ops.rs`
- Now returns `Result<DiagramProjection, ProjectionError>`
- Uses `validate_dimensions` helper for NaN/Infinity/positive checks
- Line count: 16 lines (body only)

```rust
pub fn apply_node_resize(
    state: DiagramProjection,
    id: &NodeId,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<DiagramProjection, ProjectionError> {
    validate_dimensions(width, height)?;
    let node = state.nodes.get(id).ok_or_else(|| ProjectionError::NodeNotFound(id.to_string()))?;
    let mut updated = node.clone();
    updated.x = OrderedFloat(x);
    updated.y = OrderedFloat(y);
    updated.width = OrderedFloat(width);
    updated.height = OrderedFloat(height);
    let new_nodes = state.nodes.update(id.clone(), updated);
    let new_nodes = propagate_bounds_to_ancestors(new_nodes, id);
    Ok(DiagramProjection { nodes: new_nodes, ..state })
}
```

### 3. Added validation helper
```rust
fn validate_dimensions(width: f64, height: f64) -> Result<(), ProjectionError> {
    if !width.is_finite() || width <= 0.0 {
        return Err(ProjectionError::InvalidDimensions(format!("invalid width: {width}")));
    }
    if !height.is_finite() || height <= 0.0 {
        return Err(ProjectionError::InvalidDimensions(format!("invalid height: {height}")));
    }
    Ok(())
}
```

### 4. Updated callers to convert errors
**Files**: `node_ops.rs`, `replay.rs`
- Callers use `.map_err(|e| ReplayError::InvalidEvent(e.to_string()))` to adapt to existing `apply_node_op` signature

## Contract Adherence
- ✅ EARS-3: Returns `ProjectionError::NodeNotFound` for missing nodes
- ✅ EARS-4: Returns `ProjectionError::InvalidDimensions` for NaN/Infinity
- ✅ P3/P4: Validates finite and > 0

## Files Changed
1. `diagram_tool/src/models/projection/types.rs` - Added `ProjectionError`
2. `diagram_tool/src/models/projection/ops/node_ops.rs` - Refactored function
3. `diagram_tool/src/models/projection/mod.rs` - Re-export `ProjectionError`
4. `diagram_tool/src/models/projection/replay.rs` - Error conversion
