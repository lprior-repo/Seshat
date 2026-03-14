# Black Hat Review: seshat-agw (EDG-017 to EDG-021)

## Contract Review

### Preconditions
- [P1] Edge has valid source/target node references ✓ (validated at render time)
- [P2] Nodes have valid position data ✓ (Node struct always has x, y, width, height)
- [P3] Document contains referenced nodes ✓ (checked via doc.nodes.get())

### Postconditions
- [Q1] Edge follows source node on move ✓ (dynamic render)
- [Q2] Edge follows target node on move ✓ (dynamic render)
- [Q3] Edge endpoints remain attached ✓ (center-to-center)

### Implementation Analysis

#### Code Location: `diagram_tool/src/ui/canvas/canvas_view.rs`
```rust
pub(super) fn edge_path(sx: f64, sy: f64, tx: f64, ty: f64, edge: &Edge) -> String
```
- Takes coordinates directly, no caching
- Called with current node positions from document

#### Code Location: `diagram_tool/src/ui/canvas.rs` (lines 2301-2304)
```rust
edge_rows.into_iter().map(move |(id, edge, src, tgt)| {
    let (sx, sy) = to_screen_coords(src.x.0 + src.width.0 / 2.0, src.y.0 + src.height.0 / 2.0, ...);
    let (tx, ty) = to_screen_coords(tgt.x.0 + tgt.width.0 / 2.0, tgt.y.0 + tgt.height.0 / 2.0, ...);
    let d = edge_path(sx, sy, tx, ty, &edge);
```
- Dynamically fetches current node positions at render time
- No position caching - always current

### Quality Gates

- [x] No panics in edge rendering code
- [x] No unwrap/expect in path calculation
- [x] Error handling for missing nodes (graceful skip)
- [x] Tests pass (440 library tests)
- [x] Contract satisfied by existing implementation

## STATUS: APPROVED

The feature is already implemented correctly. Edge paths are computed dynamically from current node positions at render time, satisfying all contract requirements.
