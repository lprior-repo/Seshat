# Contract Specification: bd-34z - Snap/Alignment Tests

## Bead Metadata

- **Bead ID**: bd-34z
- **Title**: snap-align: Implement snap/alignment tests (SNP-001 to SNP-010)
- **Category**: Functional Implementation
- **Priority**: High
- **Status**: In Progress

## Overview

This bead implements comprehensive snap and alignment functionality for the diagram tool, covering 10 test scenarios (SNP-001 through SNP-010). The implementation ensures zero panics/unwraps in production code and provides robust snap/alignment behavior.

## Design by Contract

### Preconditions (P1-P10)

**P1 - Grid Size Valid**
- `grid_size` parameter must be positive (> 0.0)
- Violation: Returns original position unchanged

**P2 - Snap Threshold Valid**
- `threshold` parameter must be non-negative (>= 0.0)
- Violation: Returns original position unchanged

**P3 - Node List Non-Empty**
- For multi-node snap, node list must contain at least 2 nodes
- Violation: Returns empty result set

**P4 - Guide Lines Valid**
- Guide coordinates must be finite (not NaN or Infinity)
- Violation: Filters out invalid guides

**P5 - Bounding Box Valid**
- Node bounding boxes must have positive dimensions
- Violation: Skips invalid nodes

**P6 - Alignment Anchor Valid**
- Alignment anchor must be one of: left, center, right, top, middle, bottom
- Violation: Returns error

**P7 - Snap Toggle State**
- Snap toggle must be boolean state
- Violation: Defaults to enabled

**P8 - Drag State Valid**
- Drag operation must have valid start and current positions
- Violation: No snap applied

**P9 - Resize Handle Valid**
- Resize handle must be one of: n, s, e, w, ne, nw, se, sw
- Violation: Returns error

**P10 - Distribution Count Valid**
- For distribution, at least 3 nodes must be selected
- Violation: Returns error

### Invariants (I1-I5)

**I1 - Zero Unwrap/Panic**
- All production code uses `?` operator or pattern matching
- No `.unwrap()`, `.expect()`, or `.panic!()` in production paths
- Test code may use unwrap for test assertions only

**I2 - Position Preservation**
- When snap doesn't apply, original position is preserved
- No mutation without explicit snap action

**I3 - Finite Coordinates**
- All returned coordinates are finite (not NaN or Infinity)
- Ensured via validation on input and output

**I4 - Deterministic Behavior**
- Same inputs always produce same outputs
- No randomness in snap calculations

**I5 - Transaction Safety**
- Snap operations are atomic - all or nothing
- Partial updates never committed to document

### Postconditions (Q1-Q10)

**Q1 - SNP-001: Snap to Grid**
- Given: Node at arbitrary position, grid_size > 0, snap enabled
- When: `snap_to_grid(node, grid_size)` called
- Then: Node position rounded to nearest grid intersection
- Output: `(x, y)` where `x % grid_size == 0` and `y % grid_size == 0`

**Q2 - SNP-002: Snap to Guides**
- Given: Node position, guide lines at specific coordinates
- When: `snap_to_guides(node, guides, threshold)` called
- Then: If distance to guide <= threshold, snap to guide
- Output: Snapped position or original if outside threshold

**Q3 - SNP-003: Snap to Other Nodes**
- Given: Active node, set of other nodes with positions
- When: `snap_to_nodes(active, others, threshold)` called
- Then: Snaps to edges/centers of other nodes within threshold
- Output: Snapped position with priority to closest target

**Q4 - SNP-004: Alignment Tools**
- Given: Set of selected nodes, alignment anchor
- When: `align_nodes(nodes, anchor)` called
- Then: All nodes aligned to anchor edge/center
- Output: Vector of aligned positions

**Q5 - SNP-005: Distribution Tools**
- Given: Set of selected nodes (>= 3), distribution axis
- When: `distribute_nodes(nodes, axis)` called
- Then: Nodes evenly spaced along axis
- Output: Vector of distributed positions

**Q6 - SNP-006: Snap Threshold**
- Given: Node position, snap targets, threshold value
- When: Distance check performed
- Then: Snap only if `distance <= threshold`
- Output: Boolean indicating snap applied

**Q7 - SNP-007: Snap During Drag**
- Given: Drag operation in progress, snap enabled
- When: Drag position updated
- Then: Real-time snap preview and final snap on release
- Output: Tuple of (preview_pos, final_pos)

**Q8 - SNP-008: Snap During Resize**
- Given: Resize operation in progress, snap enabled
- When: Resize handle dragged
- Then: Dimensions snap to grid/guides
- Output: Snapped dimensions

**Q9 - SNP-009: Multi-Node Snap**
- Given: Multiple selected nodes being dragged
- When: Drag position updated
- Then: All nodes snap relative to primary selection
- Output: Vector of snapped positions

**Q10 - SNP-010: Snap Toggle**
- Given: Snap enabled state, toggle action
- When: Toggle triggered
- Then: Snap state flipped, UI updated
- Output: Boolean new state

## Test Scenarios

### SNP-001: Snap to Grid
```rust
// TC1: Basic grid snap
assert_eq!(snap_to_grid((47, 53), 10), (50, 50))

// TC2: Already on grid
assert_eq!(snap_to_grid((50, 100), 10), (50, 100))

// TC3: Negative coordinates
assert_eq!(snap_to_grid((-47, -53), 10), (-50, -50))

// TC4: Half-grid offset
assert_eq!(snap_to_grid((45, 45), 10), (50, 50))
```

### SNP-002: Snap to Guides
```rust
// TC1: Snap to horizontal guide
assert_eq!(snap_to_guides((100, 52), &[], 5.0), Some((100, 50)))

// TC2: Snap to vertical guide
assert_eq!(snap_to_guides((102, 100), &[], 5.0), Some((100, 100)))

// TC3: Outside threshold
assert_eq!(snap_to_guides((100, 60), &[], 5.0), None)

// TC4: Multiple guides, pick closest
assert_eq!(snap_to_guides((100, 52), &[], 10.0), Some((100, 50)))
```

### SNP-003: Snap to Other Nodes
```rust
// TC1: Snap to left edge
assert_eq!(snap_to_nodes((110, 100), nodes, 10.0), Some((100, 100)))

// TC2: Snap to center
assert_eq!(snap_to_nodes((145, 100), nodes, 10.0), Some((150, 100)))

// TC3: Snap to right edge
assert_eq!(snap_to_nodes((188, 100), nodes, 10.0), Some((200, 100)))

// TC4: No snap - too far
assert_eq!(snap_to_nodes((150, 150), nodes, 10.0), None)
```

### SNP-004: Alignment Tools
```rust
// TC1: Align left
assert_eq!(align_left(nodes), vec![(0, 100), (0, 200), (0, 300)])

// TC2: Align center
assert_eq!(align_center(nodes), vec![(50, 100), (50, 200), (50, 300)])

// TC3: Align right
assert_eq!(align_right(nodes), vec![(100, 100), (100, 200), (100, 300)])

// TC4: Empty selection - no change
assert_eq!(align_left(vec![]), vec![])
```

### SNP-005: Distribution Tools
```rust
// TC1: Distribute horizontally
assert_eq!(distribute_h(nodes), vec![(0, 100), (100, 200), (200, 300)])

// TC2: Distribute vertically
assert_eq!(distribute_v(nodes), vec![(100, 0), (100, 100), (100, 200)])

// TC3: Too few nodes - error
assert!(distribute_h(vec![node1, node2]).is_err())

// TC4: Maintain order
assert_eq!(distribute_h(nodes).unwrap().len(), 3)
```

### SNP-006: Snap Threshold
```rust
// TC1: Within threshold
assert!(should_snap(5.0, 10.0) == true)

// TC2: Exactly at threshold
assert!(should_snap(10.0, 10.0) == true)

// TC3: Outside threshold
assert!(should_snap(11.0, 10.0) == false)

// TC4: Zero threshold
assert!(should_snap(0.0, 0.0) == true)
```

### SNP-007: Snap During Drag
```rust
// TC1: Drag with snap preview
let (preview, final) = drag_with_snap((47, 53), 10.0)
assert_eq!(preview, (50, 50))
assert_eq!(final, (50, 50))

// TC2: Drag without snap (disabled)
let (preview, final) = drag_with_snap_disabled((47, 53), 10.0)
assert_eq!(preview, (47, 53))
assert_eq!(final, (47, 53))

// TC3: Multi-node drag preserves offsets
let results = drag_multi_with_snap(nodes, (10, 10), 10.0)
assert_eq!(results[0], (50, 100))
assert_eq!(results[1], (150, 200))
```

### SNP-008: Snap During Resize
```rust
// TC1: Resize width snaps to grid
let (w, h) = resize_with_snap((80, 40), (10, 0), 10.0)
assert_eq!(w, 90)
assert_eq!(h, 40)

// TC2: Resize from different handle
let (w, h) = resize_with_snap((80, 40), (-10, 0), 10.0, "w")
assert_eq!(w, 70)

// TC3: Aspect ratio lock with snap
let (w, h) = resize_with_aspect_lock((80, 40), (20, 0), 10.0, true)
assert_eq!(w, 90)
assert_eq!(h, 45)
```

### SNP-009: Multi-Node Snap
```rust
// TC1: All nodes snap together
let results = snap_multi_nodes(nodes, (10, 10), 10.0)
assert!(results.iter().all(|p| p.0 % 10.0 == 0))

// TC2: Maintain relative positions
let offsets = calculate_offsets(nodes, snapped)
assert_eq!(offsets[0], offsets[1])

// TC3: Primary selection determines snap
let primary = nodes[0]
let results = snap_multi_to_primary(nodes, primary, 10.0)
assert_eq!(results[0].0 % 10.0, 0)
```

### SNP-010: Snap Toggle
```rust
// TC1: Toggle from disabled to enabled
let state = toggle_snap(false)
assert_eq!(state, true)

// TC2: Toggle from enabled to disabled
let state = toggle_snap(true)
assert_eq!(state, false)

// TC3: Query current state
assert_eq!(is_snap_enabled(true), true)

// TC4: Toggle during drag commits at current position
let (pos, committed) = toggle_during_drag((47, 53), false, 10.0)
assert_eq!(pos, (47, 53))
assert_eq!(committed, false)
```

## Error Handling

All functions return `Result<T, E>` or `Option<T>`:

```rust
pub enum SnapError {
    InvalidGridSize,
    InvalidThreshold,
    InvalidNodeList,
    InvalidAlignmentAnchor,
    InvalidResizeHandle,
    InsufficientNodesForDistribution,
}
```

## Performance Requirements

- Snap calculations: O(1) for grid, O(n) for nodes/guides where n = target count
- Alignment: O(n) where n = selected node count
- Distribution: O(n log n) for sorting by position
- No allocation for single-node snap
- Stack allocation for temp calculations

## Integration Points

1. **Geometry Module**: Uses existing `snap_to_grid`, `snap_horizontal`, `snap_vertical`
2. **UI Layer**: Snap preview during drag/resize
3. **Mutation Pipeline**: Atomic snap operations
4. **Document State**: Persistent snap toggle state

## Success Criteria

1. All 10 test categories (SNP-001 to SNP-010) implemented
2. Zero `unwrap()`, `expect()`, `panic!()` in production code
3. All tests pass with `cargo test`
4. No clippy warnings in snap/alignment code
5. Comprehensive error handling with actionable messages
6. Performance benchmarks meet requirements
