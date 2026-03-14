# Martin Fowler Test Plan: Moving Bound Nodes (EDG-017 to EDG-021)

## Happy Path Tests
- `test_edge_follows_source_node_on_move` - Given an edge E1 connected to nodes N1→N2, when N1 is moved, then E1's rendered path reflects N1's new position
- `test_edge_follows_target_node_on_move` - Given an edge E1 connected to nodes N1→N2, when N2 is moved, then E1's rendered path reflects N2's new position
- `test_edge_follows_both_nodes_on_move` - Given an edge E1 connected to nodes N1→N2, when both nodes are moved, then E1's rendered path reflects both new positions

## Error Path Tests
- `test_returns_none_for_edge_with_missing_source_node` - Given an edge with non-existent source node ID, when rendering, then the edge is skipped/not rendered
- `test_returns_none_for_edge_with_missing_target_node` - Given an edge with non-existent target node ID, when rendering, then the edge is skipped/not rendered

## Edge Case Tests
- `test_edge_with_zero_dimension_node` - Given a node with width=0 or height=0, when edge is rendered, then path calculation doesn't panic
- `test_edge_between_adjacent_nodes` - Given two nodes at the same position, when edge is rendered, then path degenerates to a point without error
- `test_multiple_edges_same_node` - Given multiple edges connected to the same node, when that node moves, then all edges update correctly

## Contract Verification Tests
- `test_precondition_source_node_exists` - Verify edge has valid source node reference
- `test_precondition_target_node_exists` - Verify edge has valid target node reference
- `test_postcondition_path_uses_current_node_positions` - After node move, edge path coordinates match node positions

## Contract Violation Tests
- `test_violates_p1_missing_source_node_handled_gracefully` - Given edge with source="non-existent", when rendering, then edge is skipped without panic
- `test_violates_p2_node_with_invalid_position_returns_error` - Given node with NaN position, when edge is rendered, then returns Error::NodePositionUnavailable
- `test_violates_p3_deleted_node_reference_handled` - Given edge referencing node deleted from document, when rendering, then edge is skipped
- `test_violates_q1_stale_render_detected` - Move node N1, verify edge path uses new position not old
- `test_violates_q2_cached_positions_not_used` - Verify edge path uses current node positions, not cached
- `test_violates_q3_endpoints_remain_attached` - After node move, edge endpoints still connect to node centers

## Given-When-Then Scenarios

### Scenario 1: Source node moved horizontally
**Given**: Document with node N1 at (0, 0) and node N2 at (100, 0), connected by edge E1  
**When**: N1 is moved to position (50, 0)  
**Then**: Edge E1 renders from (50, 0) to (100, 0) - path updated

### Scenario 2: Target node moved vertically
**Given**: Document with node N1 at (0, 0) and node N2 at (100, 0), connected by edge E1  
**When**: N2 is moved to position (100, 50)  
**Then**: Edge E1 renders from (0, 0) to (100, 50) - path updated

### Scenario 3: Both nodes moved simultaneously
**Given**: Document with node N1 at (0, 0) and node N2 at (100, 0), connected by edge E1  
**When**: N1 is moved to (25, 25) and N2 is moved to (75, 75)  
**Then**: Edge E1 renders from (25, 25) to (75, 75) - path updated

### Scenario 4: Edge with missing source node
**Given**: Edge E1 referencing source node "non-existent" → N2  
**When**: Document is rendered  
**Then**: Edge E1 is skipped, no render error occurs

### Scenario 5: Multiple edges connected to moved node
**Given**: Three edges E1, E2, E3 all connected to node N1 as source  
**When**: N1 is moved to a new position  
**Then**: All three edges E1, E2, E3 render with updated source position
