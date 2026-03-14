# Martin Fowler Test Plan

bead_id: seshat-idr
bead_title: SUB-023 to SUB-027: Subgraph relative coordinates
phase: p1
updated_at: 2026-03-14T18:00:00Z

## Happy Path Tests
- `test_sub023_child_coords_relative_to_parent`: Verify child stored x/y are relative.
- `test_sub024_moving_parent_updates_child_world_coords`: Verify child world position changes when parent moves.
- `test_sub025_nesting_multiple_levels`: Verify relative coordinates work across multi-level nesting.
- `test_sub026_reparenting_preserves_world_position`: Verify `keep_world_pos` flag updates relative coords correctly.
- `test_sub027_root_node_is_world_space`: Verify nodes with `parent == None` use world space.

## Error Path Tests
- `test_error_orphan_child_parent_not_found`: Verify `get_world_coords` fails if parent is missing from map.
- `test_error_reparenting_cycle`: Verify cycle detection works.

## Edge Case Tests
- `test_edge_zero_size_parent`: Verify relative coords work even if parent width/height is 0.
- `test_edge_negative_relative_coords`: Verify children can be positioned outside parent bounds (negative relative coords).

## Contract Verification Tests
- `test_precondition_valid_parent_existence`
- `test_invariant_world_coord_calculation`

## Contract Violation Tests
- `test_p2_violation_returns_node_not_found`:
  Given: A node with a parent ID that doesn't exist.
  When: `get_world_coords` is called.
  Then: returns `Err(Error::NodeNotFound)`.

## Given-When-Then Scenarios
### Scenario 1: SUB-024 Moving Parent
Given: A parent node at (10, 10) and a child at (5, 5) relative to parent.
When: Parent is moved to (20, 20).
Then:
- Child's stored coordinates remain (5, 5).
- Child's world coordinates become (25, 25).

### Scenario 2: SUB-026 Reparenting
Given: Node A at world (100, 100), Parent P at world (50, 50).
When: Node A is reparented to P with `keep_world_pos = true`.
Then:
- Node A's parent is now P.
- Node A's stored coordinates become (50, 50) -- since 50+50 = 100.
