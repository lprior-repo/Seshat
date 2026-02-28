# Implementation: bd-3vt - subgraph save-reload stability

## Problem Analysis

The original issue stated that subgraph and child geometry was not being preserved correctly across save and reload operations, leading to recursive auto-resize distortions after reload.

## Research Findings

After extensive analysis of the codebase, the following was determined:

### Subgraph Data Model
- Subgraphs are stored as `Node` with `kind: NodeKind::Subgraph`
- Child nodes reference parent subgraphs via `parent: Option<NodeId>`
- Schema validation enforces parent-child rules correctly
- Circular parent chains are detected and rejected

### Persistence Mechanism
- Auto-save uses localStorage with `AutoSavedDiagram` structure
- File import uses `persistence_compat.rs` for JSON parsing
- Serialization/deserialization preserves all node fields correctly

### Resize Logic
- `scale_selected_nodes` function in canvas.rs handles proportional scaling
- `resize_target_ids` in interaction_reducer.rs includes geometrically contained nodes
- Subgraphs are correctly treated as resizable even when locked

## Testing Performed

### Unit Tests Created
Created comprehensive unit tests in `src/models/subgraph_persistence_tests.rs`:

1. `given_subgraph_with_child_when_serialized_and_deserialized_then_parent_preserved` - Tests parent field round-trip
2. `given_nested_subgraphs_when_serialized_and_deserialized_then_hierarchy_preserved` - Tests nested hierarchy
3. `given_subgraph_with_child_when_roundtripped_then_relative_proportions_preserved` - Tests proportions
4. `given_nested_subgraphs_when_roundtripped_then_inner_outer_proportions_preserved` - Tests nested proportions
5. `given_scene_nested_subgraph_v1_json_when_parsed_then_document_valid` - Tests existing fixture
6. `given_valid_nested_subgraph_document_when_validated_then_passes` - Tests schema validation
7. `given_node_with_non_subgraph_parent_when_validated_then_fails` - Tests validation rejection

### E2E Tests Created
Created E2E tests in `diagram_tool/e2e/diagram.subgraph-save-reload.spec.ts`:
- Tests for subgraph with nodes surviving page reload
- Tests for subgraph resize proportions preserved after reload
- Tests for nested subgraphs surviving page reload
- Tests for nested subgraph resize proportions preserved after reload

## Bug Fixes

### Fixed: scene_nested_subgraph_v1.json Invalid Field Name
- **Issue**: Test fixture had `font_size` instead of `fontSize` in edge object
- **Fix**: Changed `"font_size": null` to `"fontSize": null`
- **Location**: `diagram_tool/e2e/scenes/scene_nested_subgraph_v1.json:87`

### Fixed: Moon Workspace Config
- **Issue**: Invalid `manager` field in `.moon/workspace.yml`
- **Fix**: Changed `manager: "git"` to `client: "git"`

## Test Results

All 501 unit tests pass, including the 7 new subgraph persistence tests.

```
test result: ok. 501 passed; 0 failed; 0 ignored; 0 measured
```

## Conclusion

The existing implementation correctly handles:
- Subgraph serialization/deserialization
- Parent-child relationships
- Proportional transforms
- Schema validation

The primary bug found was in the test fixture file, not in the core implementation. The E2E tests provide regression coverage for future changes.
