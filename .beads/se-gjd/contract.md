# Contract: se-gjd - Test Infrastructure for 240 Test Cases

## Status: Draft

## Overview

Set up the comprehensive test infrastructure for all 240 test cases across 12 categories. This provides the scaffolding, fixtures, harness, and category-organized test modules that future beads will populate with actual test implementations.

## Current State Assessment

### What Already Exists
- `diagram_tool/src/test_utils/` - Full test infrastructure with builders, fixtures, generators, harness, and types
- `diagram_tool/src/test_utils/builders/` - NodeBuilder, EdgeBuilder, DocBuilder (fluent pattern)
- `diagram_tool/src/test_utils/fixtures.rs` - Golden scene loading, schema validation, operation snapshots
- `diagram_tool/src/test_utils/generators.rs` - Golden scene creation, fuzz testing, stress scene generation (5000 nodes)
- `diagram_tool/src/test_utils/harness.rs` - Invariant verification, test runner, test DB paths
- `diagram_tool/src/test_utils/types.rs` - TestCategory enum (11 categories), error taxonomy, report types
- `diagram_models/src/test_utils.rs` - Legacy test builders (NodeBuilder, EdgeBuilder, DocBuilder)
- `diagram_tool/tests/fixtures/` - 13 golden scene JSON fixtures
- `canvas_math/src/proptests.rs` - Property-based tests (256 cases each)
- `canvas_math/src/kani_proofs.rs` - Formal verification proofs

### Gap: Category-Organized Test Modules
The test_utils infrastructure has a TestCategory enum listing 11 categories with 228 total tests. The bead spec calls for 240 tests across 12 categories. We need to:
1. Add the DOC category (Document tests) and PERF category (Performance tests) to reach 12 categories
2. Create dedicated test module structure under `diagram_tool/src/tests/` for each category
3. Add proptest strategies for diagram-specific property-based testing
4. Create a nextest.toml configuration for structured test execution

## Architecture

### Test Category Map (240 tests, 12 categories)

| Category | Code | Count | Location | Focus |
|----------|------|-------|----------|-------|
| Document | DOC | 12 | tests/doc/ | Serialization, schema, versioning, round-trip |
| Geometry | GEO | 30 | tests/geo/ | AABB, transforms, intersections, bounds |
| Selection | SEL | 25 | tests/sel/ | Click, marquee, multi-select, deselect |
| Multi-select | MUL | 37 | tests/mul/ | Drag, rotate, scale, align, distribute |
| Subgraph | SUB | 34 | tests/sub/ | Create, destroy, reparent, cascade, transform |
| Edge Binding | EDG | 35 | tests/edg/ | Route, bend, label, port, direction |
| Viewport | CAM | 12 | tests/cam/ | Pan, zoom, fit, screen/canvas transforms |
| Snap/Align | SNP | 10 | tests/snp/ | Grid snap, alignment, distribution |
| Clipboard | CLP | 10 | tests/clp/ | Copy, paste, cut, cross-document |
| History | HIS | 13 | tests/his/ | Undo, redo, branch, merge, eviction |
| IO | IO | 15 | tests/io/ | Import, export, SVG, PNG, JSON schema |
| Performance | PERF | 7 | tests/perf/ | Large docs, rendering, memory, regression |

Total: 240

### Test Module Structure

```
diagram_tool/src/tests/
    mod.rs              -- Test module registry, re-exports
    doc_tests.rs        -- DOC: Document serialization and schema
    geo_tests.rs        -- GEO: Geometry calculations
    sel_tests.rs        -- SEL: Selection operations
    mul_tests.rs        -- MUL: Multi-select transforms
    sub_tests.rs        -- SUB: Subgraph operations
    edg_tests.rs        -- EDG: Edge binding and routing
    cam_tests.rs        -- CAM: Viewport/camera
    snp_tests.rs        -- SNP: Snap and alignment
    clp_tests.rs        -- CLP: Clipboard operations
    his_tests.rs        -- HIS: History (undo/redo)
    io_tests.rs         -- IO: Import/Export
    perf_tests.rs       -- PERF: Performance regression
    proptest_strategies.rs -- Shared proptest generators
```

### Proptest Strategies

Located in `tests/proptest_strategies.rs`:
- `arb_node()` - Generates random valid Node instances
- `arb_edge()` - Generates random valid Edge instances (requires node IDs)
- `arb_document()` - Generates random valid DiagramDocument instances
- `arb_zoom()` - Generates valid zoom values (0.1..4.0)
- `arb_point()` - Generates valid 2D points
- `arb_rect()` - Generates valid rectangles (positive dimensions)
- `arb_node_kind()` - Generates NodeKind variants
- `arb_selection()` - Generates selection sets

### Fixture Strategy

Existing fixtures in `tests/fixtures/` cover:
- mixed_selection.json - Multi-node selection scenarios
- nested_subgraph.json - Subgraph containment
- group_before/after.json - Grouping operations
- move_before/after.json - Move operations
- resize_before/after.json - Resize operations
- reparent_before/after.json - Reparent operations
- rotate_before/after.json - Rotation operations
- perf/small_scene.json - Performance baseline

New fixtures needed:
- doc_round_trip.json - Full serialization test
- edge_routing.json - Complex edge routing
- snap_grid.json - Snap alignment test
- history_stack.json - Deep undo/redo stack
- import_export.json - IO round-trip data

### nextest Configuration

`.nextest.toml`:
- Test timeout: 60s (default), 300s (proptest), 600s (stress)
- Test groups for parallelism control
- Retry policy for flaky test mitigation

## Design by Contract

### Preconditions
- P1: All test modules compile with `#[cfg(test)]`
- P2: Test infrastructure does not import WASM-incompatible crates
- P3: No `unwrap`/`panic`/`expect` in test infrastructure code (only in test bodies)
- P4: Each test module has at least one test verifying the module loads

### Postconditions
- Q1: `diagram_tool/src/tests/` contains 14 files (mod.rs + 12 category + proptest_strategies)
- Q2: Total test stub count sums to 240
- Q3: `nextest.toml` configured for structured execution
- Q4: All infrastructure compiles and passes `moon run :ci-source`

### Invariants
- I1: No test infrastructure code leaks into production builds (`#[cfg(test)]`)
- I2: All fixtures use schema version 2
- I3: proptest strategies only generate valid domain objects
- I4: Test modules are independent (no cross-module test dependencies)

## Implementation Plan

1. Update TestCategory enum to include Doc and Perf (12 categories, 240 total)
2. Create `diagram_tool/src/tests/` directory with all 14 files
3. Each category file contains placeholder tests with proper structure
4. Add proptest strategies for domain objects
5. Create `.nextest.toml` configuration
6. Register the tests module in `lib.rs`
7. Verify compilation and CI pass
