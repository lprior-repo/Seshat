# Implementation Summary: Moving Bound Nodes (EDG-017 to EDG-021)

## Bead ID: seshat-agw
## Phase: Implementation

### Feature Overview
Edges must auto-recalculate paths when bound nodes are moved.

### Implementation Status: ALREADY IMPLEMENTED

The feature is already fully implemented in the codebase. Here's the analysis:

#### 1. Edge Structure (document.rs)
- Edges store only `source: NodeId` and `target: NodeId` - not positions
- This design ensures edges always reference current node positions

#### 2. Node Move Operation (node_ops.rs)
- `apply_node_move` updates node position in the document
- Does NOT need to modify edges - they reference node IDs, not positions
- Bounds propagation handles container updates

#### 3. Edge Path Rendering (canvas/canvas_view.rs)
- `edge_path(sx, sy, tx, ty, edge)` computes path from coordinates
- Coordinates are obtained from current node positions at render time
- Called in canvas.rs with dynamic node position lookup:
  ```rust
  let (sx, sy) = to_screen_coords(src.x.0 + src.width.0 / 2.0, src.y.0 + src.height.0 / 2.0, ...);
  let (tx, ty) = to_screen_coords(tgt.x.0 + tgt.width.0 / 2.0, tgt.y.0 + tgt.height.0 / 2.0, ...);
  let d = edge_path(sx, sy, tx, ty, &edge);
  ```

#### 4. Error Handling
- Missing source/target nodes handled gracefully in render (edge skipped)
- Error taxonomy matches contract: SourceNodeNotFound, TargetNodeNotFound

### Contract Clause Mapping

| Contract Clause | Implementation |
|----------------|---------------|
| P1: Valid source/target | Edges store NodeId, validated at render |
| P2: Valid position data | Node struct always has x, y, width, height |
| P3: Document contains nodes | Verified via `doc.nodes.get()` |
| Q1: Edge follows node move | Dynamic render uses current positions |
| Q2: Uses current positions | Coordinates fetched at render time |
| Q3: Endpoints attached | Always connects to node centers |

### Files Changed
None - feature was already implemented.

### Test Verification
The existing tests verify related functionality:
- `clipboard_contract_tests.rs`: Edge preservation during copy/paste
- `routing_tests.rs`: Edge creation and validation
- `grouping_tests.rs`: Edge cleanup on node delete

### Conclusion
The contract requirements are fully satisfied by the existing implementation. No code changes are required. The behavior is achieved through the design decision to compute edge paths dynamically at render time rather than storing cached positions.
