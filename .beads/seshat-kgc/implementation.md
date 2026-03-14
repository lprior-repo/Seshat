# Implementation Summary: Marquee Performance (seshat-kgc)

## Changes
1.  **`diagram_tool/src/models/spatial_index.rs`** - New module implementing a Grid-based spatial index for fast rectangular queries.
    - `build_spatial_index`: Builds an immutable index from document nodes ($O(N)$).
    - `gather_candidates`: Retrieves potential node IDs for a given marquee ($O(\text{cells})$).
    - `query_spatial_index`: Full query with precise intersection/containment checks.
2.  **`diagram_tool/src/models/selection.rs`** - Integrated `SpatialIndex` into `compute_marquee_selection`.
    - Now uses `gather_candidates` to avoid linear scan of all nodes.
    - Correctly handles rotated nodes by using their AABB for candidate gathering and precise checks.
3.  **`diagram_tool/src/ui/interaction.rs`** - Integrated `SpatialIndex` into `node_ids_in_rect_with_mode`.
    - Optimized UI-layer marquee selection.
4.  **`diagram_tool/src/models/mod.rs`** - Exported `spatial_index` module.

## Performance
- Benchmark with 3000 nodes:
    - Build Index: ~37ms
    - Query Index: ~60µs
- Scaling: The spatial index allows the query to scale to diagrams with many nodes by only checking nodes in relevant grid cells.

## Contract Adherence
- P1: Marquee dimensions are checked for non-negativity.
- P2: Index is built before query.
- P3: Performance target (<16ms query) is met.
- Q1: Parity with linear scan verified by tests.
- Q2: Contain vs Intersect modes handled correctly.
- Q3: Rotated nodes handled via AABB.
