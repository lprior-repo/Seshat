# Martin Fowler Test Plan

## Happy Path Tests
- **test_multi_select_drag_into_container_reparents_all_nodes**
  Given: Container A with children X, Y, and node B outside container
  And: Nodes X and Y are selected (multi-select)
  When: User drags X and Y into container A
  Then: X and Y become children of container A

- **test_multi_select_drag_across_boundary_maintains_world_transform**
  Given: Container at (100, 100) with size (200, 200)
  And: Selected node at (120, 120) inside container after drag
  When: Node is reparented to container
  Then: Node remains at (120, 120) visually

- **test_multi_select_drag_out_of_container_reparents_to_root**
  Given: Container A with selected children X, Y
  When: User drags X and Y outside container A
  Then: X and Y become root-level nodes (parent = None)

## Error Path Tests
- **test_multi_select_cannot_reparent_to_own_descendant**
  Given: Container A with child B
  And: Node B is selected
  When: User attempts to drag B onto its descendant (impossible in DAG)
  Then: Operation is rejected, no reparent occurs

- **test_multi_select_cannot_reparent_container_in_selection**
  Given: Container A and its child B
  And: Both A and B are selected
  When: User drags selection
  Then: Container A is NOT reparented (it's in the selection)

## Edge Case Tests
- **test_multi_select_drag_partial_into_container**
  Given: Container A with selected nodes X, Y inside and Z outside
  When: User drags all three nodes into container A
  Then: Only X and Y are reparented, Z remains in place

- **test_multi_select_drag_nested_containers**
  Given: Container A containing Container B
  And: Node X is child of B
  And: X is selected
  When: User drags X outside of both containers
  Then: X becomes child of A (not root)

## Contract Verification Tests
- **test_precondition_p1_selection_not_empty**
  Given: Empty selection
  When: Drag operation attempted
  Then: No reparent occurs (handled by select mode)

- **test_precondition_p3_crosses_boundary**
  Given: Selected nodes not crossing container boundary
  When: Drag ends
  Then: No reparent occurs

- **test_postcondition_q1_all_nodes_reparented**
  Given: Multi-selection dragged into container
  Then: All selected nodes have target container as parent

- **test_postcondition_q4_selection_preserved**
  Given: Multi-selection reparented
  Then: Same nodes remain selected after reparent

## Given-When-Then Scenarios

### Scenario 1: Drag multi-selection into subgraph
**Given** a canvas with Subgraph A at (0, 0) with size (300, 300)
**And** nodes X, Y inside Subgraph A at positions (50, 50) and (100, 100)
**And** nodes P, Q outside Subgraph A at positions (350, 50) and (400, 100)
**And** nodes P, Q are selected (multi-select)
**When** user drags P and Q into Subgraph A (to positions 50, 150 and 100, 200)
**Then** node P becomes child of Subgraph A
**And** node Q becomes child of Subgraph A
**And** P and Q maintain their screen positions
**And** P and Q remain selected

### Scenario 2: Drag multi-selection out of subgraph
**Given** a canvas with Subgraph A at (0, 0) with size (300, 300)
**And** nodes X, Y inside Subgraph A at positions (50, 50) and (100, 100)
**And** X and Y are selected (multi-select)
**When** user drags X and Y outside Subgraph A (to positions 400, 50 and 450, 100)
**Then** node X becomes root-level (parent = None)
**And** node Y becomes root-level (parent = None)
**And** X and Y maintain their screen positions

### Scenario 3: Undo multi-select reparent
**Given** nodes X, Y were reparented from Subgraph A to Subgraph B
**When** user presses Ctrl+Z (undo)
**Then** X and Y return to being children of Subgraph A
**And** X and Y remain selected
