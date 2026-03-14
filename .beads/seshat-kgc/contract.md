# Contract Specification: Marquee Performance (seshat-kgc)

## Context
- Feature: Optimized marquee selection for large diagrams (3000+ nodes).
- Domain terms: 
    - SpatialIndex: A data structure to accelerate rectangular queries.
    - Marquee: A rectangular selection box.
    - Query: Finding all nodes that intersect or are contained within a marquee.
- Assumptions:
    - Node positions and dimensions are provided by the `DiagramDocument`.
    - Nodes can be rotated (metadata "rotation").
    - Performance target: < 16ms for 3000 nodes on representative hardware.

## Preconditions
- P1: Marquee rectangle must have non-negative width and height.
- P2: Spatial index must be initialized with the current document state before querying.
- P3: Performance must stay below 16ms for 3000 nodes.

## Postconditions
- Q1: Result set must be identical to the O(N) linear scan result.
- Q2: Result set must handle Contain vs Intersect modes correctly.
- Q3: Rotated nodes must be correctly handled (using their AABB for the spatial index query, then precise check).

## Error Taxonomy
- Error::InvalidMarqueeBounds - when width or height is negative.
- Error::IndexNotInitialized - if query is called before index is built.
- Error::PerformanceTargetViolated - if query exceeds 16ms for 3000 nodes (debug_assert).
- Error::PostconditionViolated - if result set is incorrect.

## Contract Signatures
- `fn build_spatial_index(nodes: &HashMap<NodeId, Node>) -> SpatialIndex`
- `fn query_spatial_index(index: &SpatialIndex, marquee: Rect, mode: MarqueeMode) -> HashSet<NodeId>`

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| width >= 0, height >= 0 | Runtime-checked constructor | `Rect::new() -> Result` |
| mode | Compile-time | `enum MarqueeMode { Contain, Intersect }` |
| index initialized | Compile-time (Option/Wrapped) | `Option<SpatialIndex>` or `&SpatialIndex` |

## Violation Examples (REQUIRED -- one per precondition and postcondition)
- VIOLATES <P1>: `Rect::new(0.0, 0.0, -10.0, 10.0)` -- should produce `Err(InvalidMarqueeBounds)`
- VIOLATES <P2>: `query_spatial_index(uninitialized_index, marquee, mode)` -- should produce `Err(IndexNotInitialized)` (if Option is None)
- VIOLATES <P3>: `query_spatial_index` with 3000 nodes takes 100ms -- should produce `debug_assert!` failure or `Err(PerformanceTargetViolated)`
- VIOLATES <Q1>: `query_spatial_index` returns `["node-1"]` but linear scan returns `["node-1", "node-2"]` -- should produce `Err(PostconditionViolated)`
- VIOLATES <Q2>: `query_spatial_index(Contain)` returns nodes that only intersect -- should produce `Err(PostconditionViolated)`
- VIOLATES <Q3>: `query_spatial_index` misses a rotated node whose AABB is in the marquee -- should produce `Err(PostconditionViolated)`

## Ownership Contracts
- `build_spatial_index`: Takes `&HashMap`, does not own. Returns owned `SpatialIndex`.
- `query_spatial_index`: Takes `&SpatialIndex`, does not own. Returns owned `HashSet<NodeId>`.

## Non-goals
- Full virtualization of rendering (only focused on selection logic).
- Incremental index updates (rebuilding for now is acceptable if fast).
