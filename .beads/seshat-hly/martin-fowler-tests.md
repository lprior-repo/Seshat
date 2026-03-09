# Martin Fowler Test Plan

## Happy Path Tests

### Edge Routing Domain Tests
- `test_create_edge_between_existing_nodes_succeeds`
  Given: A document with two existing nodes (node_a at (0,0), node_b at (100,100))
  When: Creating an edge from node_a to node_b
  Then: Edge is inserted into document with correct source/target
  Then: Edge has default styling (empty label, default arrow, directed=true)

- `test_create_multiple_edges_preserves_dag_integrity`
  Given: A document with nodes A, B, C
  When: Creating edges A→B and B→C (valid DAG)
  Then: Both edges exist in document
  Then: No cycles exist in graph

- `test_delete_edge_removes_from_document`
  Given: A document with an edge A→B
  When: Deleting the edge
  Then: Edge no longer exists in document
  Then: Nodes A and B still exist

- `test_validate_edge_routing_returns_valid_route`
  Given: Two nodes with valid finite coordinates
  When: Validating edge routing
  Then: Returns Ok with computed route

### Grouping Domain Tests
- `test_group_selection_creates_subgraph_with_correct_bounds`
  Given: Two selected nodes at (100,100) and (200,200) with size 50x50
  When: Grouping selection
  Then: Subgraph created with bounds including padding (80,80) to (270,270)
  Then: Width = 190, Height = 190

- `test_group_selection_reparents_children`
  Given: Selected nodes n1 and n2
  When: Grouping selection
  Then: n1.parent == Some(group_id)
  Then: n2.parent == Some(group_id)

- `test_group_selection_updates_selection_to_group`
  Given: Multiple nodes selected (n1, n2, n3)
  When: Grouping selection
  Then: Selection contains only the new group node

- `test_ungroup_selection_removes_subgraph_and_orphans_children`
  Given: A subgraph with children c1, c2
  When: Ungrouping selection
  Then: Subgraph node deleted from document
  Then: c1.parent == None
  Then: c2.parent == None
  Then: c1 and c2 are selected

- `test_ungroup_selection_preserves_nested_parent`
  Given: Nested subgraphs: parent_group → child_group → leaf_node
  When: Ungrouping child_group
  Then: child_group deleted
  Then: leaf_node.parent == Some(parent_group)
  Then: leaf_node is selected

- `test_ungroup_selection_removes_connected_edges`
  Given: A subgraph with an external edge from subgraph to node
  When: Ungrouping selection
  Then: The connected edge is removed from document

## Error Path Tests

### Edge Routing Errors
- `test_create_edge_returns_error_when_source_missing`
  Given: Document with only target node
  When: Creating edge from nonexistent source
  Then: Returns Err(RoutingError::SourceNotFound(_))

- `test_create_edge_returns_error_when_target_missing`
  Given: Document with only source node
  When: Creating edge to nonexistent target
  Then: Returns Err(RoutingError::TargetNotFound(_))

- `test_create_edge_returns_error_on_self_loop`
  Given: Single node in document
  When: Creating edge from node to itself
  Then: Returns Err(RoutingError::SelfLoop(node_id))

- `test_create_edge_returns_error_on_cycle`
  Given: Document with edges A→B and B→C
  When: Creating edge C→A
  Then: Returns Err(RoutingError::CycleDetected)

- `test_validate_edge_routing_error_on_nan_coordinates`
  Given: Node with NaN x coordinate
  When: Validating edge routing
  Then: Returns Err(RoutingError::InvalidNodeCoordinates(_))

- `test_validate_edge_routing_error_on_infinite_coordinates`
  Given: Node with Inf x coordinate
  When: Validating edge routing
  Then: Returns Err(RoutingError::InvalidNodeCoordinates(_))

### Grouping Errors
- `test_group_selection_returns_error_on_empty_selection`
  Given: Empty editor selection
  When: Grouping selection
  Then: Returns Err(GroupingError::EmptySelection)

- `test_group_selection_returns_error_on_locked_node`
  Given: Selected node with locked=true
  When: Grouping selection
  Then: Returns Err(GroupingError::LockedNode(locked_node_id))

- `test_validate_subgraph_bounds_returns_error_when_too_small`
  Given: width=10, height=10 (below 20x20 minimum)
  When: Validating subgraph bounds
  Then: Returns Err(GroupingError::SubgraphTooSmall { width: 10.0, height: 10.0 })

- `test_ungroup_selection_returns_error_on_empty_selection`
  Given: Empty selection
  When: Ungrouping selection
  Then: Returns Err(GroupingError::EmptySelection)

- `test_ungroup_selection_returns_error_when_no_subgraph_selected`
  Given: Selected regular nodes (not subgraphs)
  When: Ungrouping selection
  Then: Returns Err(GroupingError::EmptySelection)

## Edge Case Tests

### Boundary Conditions
- `test_subgraph_bounds_at_exact_minimum_size`
  Given: width=20, height=20 (MIN_SUBGRAPH_SIZE)
  When: Validating bounds
  Then: Returns Ok with BoundingBox

- `test_subgraph_bounds_below_minimum_by_one_pixel`
  Given: width=19.9, height=20
  When: Validating bounds
  Then: Returns Err(GroupingError::SubgraphTooSmall)

- `test_group_single_node_creates_minimal_subgraph`
  Given: Single node at (100,100) with size 50x50
  When: Grouping selection
  Then: Creates valid subgraph with padding
  Then: Single child reparented

- `test_group_large_number_of_nodes`
  Given: 100 nodes at various positions
  When: Grouping selection
  Then: Creates valid subgraph containing all nodes
  Then: All nodes reparented

- `test_group_nodes_at_negative_coordinates`
  Given: Nodes at (-100, -100) and (-50, -50)
  When: Grouping selection
  Then: Creates valid subgraph (negative bounds are valid)

- `test_ungroup_subgraph_with_no_children`
  Given: Subgraph node with no children
  When: Ungrouping selection
  Then: Succeeds
  Then: Subgraph removed from document

- `test_ungroup_nested_subgraphs_deletes_only_selected`
  Given: parent_group containing child_group containing leaf
  When: Ungrouping child_group only
  Then: child_group deleted
  Then: parent_group remains
  Then: leaf orphaned

### Special Values
- `test_edge_with_very_large_coordinates`
  Given: Node at (1e10, 1e10)
  When: Creating edge
  Then: Succeeds (finite but large is valid)

- `test_subgraph_bounds_with_zero_size`
  Given: width=0, height=0
  When: Validating bounds
  Then: Returns Err(GroupingError::SubgraphTooSmall)

- `test_create_edge_with_existing_edge_id`
  Given: Document with edge_id "e1"
  When: Creating another edge with same edge_id
  Then: Edge is replaced (or error depending on design)

## Contract Verification Tests

### Precondition Verification (P1-P9)
- `test_precondition_p1_source_node_exists`
  Given: Document without source node
  When: Calling create_edge with nonexistent source
  Then: Returns Err(RoutingError::SourceNotFound)

- `test_precondition_p2_target_node_exists`
  Given: Document without target node
  When: Calling create_edge with nonexistent target
  Then: Returns Err(RoutingError::TargetNotFound)

- `test_precondition_p3_source_not_equal_to_target`
  Given: Document with node
  When: Calling create_edge with same source/target
  Then: Returns Err(RoutingError::SelfLoop)

- `test_precondition_p4_no_cycle_created`
  Given: DAG that would cycle
  When: Creating edge that creates cycle
  Then: Returns Err(RoutingError::CycleDetected)

- `test_precondition_p5_selection_not_empty`
  Given: Empty selection
  When: Calling group_selection
  Then: Returns Err(GroupingError::EmptySelection)

- `test_precondition_p6_no_locked_nodes`
  Given: Locked node in selection
  When: Calling group_selection
  Then: Returns Err(GroupingError::LockedNode)

- `test_precondition_p7_valid_node_coordinates`
  Given: Node with NaN coordinates
  When: Creating edge to that node
  Then: Returns Err(RoutingError::InvalidNodeCoordinates)

- `test_precondition_p8_min_subgraph_size`
  Given: Computed bounds of 10x10
  When: Validating subgraph bounds
  Then: Returns Err(GroupingError::SubgraphTooSmall)

- `test_precondition_p9_subgraph_selected_for_ungroup`
  Given: Non-subgraph nodes selected
  When: Calling ungroup_selection
  Then: Returns Err(GroupingError::EmptySelection)

### Postcondition Verification (Q1-Q9)
- `test_postcondition_q1_edge_inserted`
  Given: Valid edge creation
  When: After create_edge succeeds
  Then: doc.document.edges.contains_key(edge_id) == true

- `test_postcondition_q2_edge_has_correct_source_target`
  Given: Valid edge creation
  When: After create_edge succeeds
  Then: edge.source == source
  Then: edge.target == target

- `test_postcondition_q2_edge_has_default_styling`
  Given: Valid edge creation
  When: After create_edge succeeds
  Then: edge.label == ""
  Then: edge.directed == true
  Then: edge.bend_points.is_empty()

- `test_postcondition_q3_subgraph_created`
  Given: Valid group_selection
  When: After group_selection succeeds
  Then: doc.document.nodes.contains_key(group_id)

- `test_postcondition_q3_subgraph_has_correct_kind`
  Given: Valid group_selection
  When: After group_selection succeeds
  Then: node.kind == NodeKind::Subgraph

- `test_postcondition_q4_children_reparented`
  Given: Valid group_selection with selected nodes n1, n2
  When: After group_selection succeeds
  Then: doc.document.nodes.get(&n1).parent == Some(group_id)
  Then: doc.document.nodes.get(&n2).parent == Some(group_id)

- `test_postcondition_q5_selection_updated`
  Given: Valid group_selection
  When: After group_selection succeeds
  Then: doc.editor_state.selected_items.len() == 1
  Then: doc.editor_state.selected_items.contains(group_id.as_str())

- `test_postcondition_q6_subgraph_removed`
  Given: Valid ungroup_selection
  When: After ungroup_selection succeeds
  Then: doc.document.nodes.get(subgraph_id) == None

- `test_postcondition_q7_children_orphaned`
  Given: Valid ungroup_selection
  When: After ungroup_selection succeeds
  Then: All former children have parent == None or inherited parent

- `test_postcondition_q8_edges_removed`
  Given: Valid ungroup_selection with connected edges
  When: After ungroup_selection succeeds
  Then: No edges reference deleted subgraph nodes

- `test_postcondition_q9_orphans_selected`
  Given: Valid ungroup_selection
  When: After ungroup_selection succeeds
  Then: All orphaned children in editor_state.selected_items

### Invariant Verification (I1-I7)
- `test_invariant_i1_dag_remains_acyclic`
  Given: Valid document
  When: After multiple edge creations
  Then: validate_dag returns Ok

- `test_invariant_i2_no_dangling_edges`
  Given: Document after edge creation
  When: Checking all edges
  Then: For all edges, source and target exist in nodes

- `test_invariant_i3_children_within_parent_bounds`
  Given: Document with parent-child relationships
  When: Checking all child nodes
  Then: Child bounds are within parent bounds

- `test_invariant_i4_all_coordinates_finite`
  Given: Any document state
  When: Checking all nodes
  Then: node.x.is_finite() == true
  Then: node.y.is_finite() == true

- `test_invariant_i5_nodeid_nonempty`
  Given: Any NodeId
  When: Checking inner string
  Then: string.len() > 0

- `test_invariant_i6_edgeid_nonempty`
  Given: Any EdgeId
  When: Checking inner string
  Then: string.len() > 0

## Contract Violation Tests

### P1-P4 Violations (Edge Creation)
- `test_violates_p1_source_not_found_returns_error`
  Given: Document with node "target" but no "source"
  When: create_edge(&mut doc, NodeId::new("source"), NodeId::new("target"), edge_id)
  Then: Returns Err(RoutingError::SourceNotFound(NodeId::new("source")))

- `test_violates_p2_target_not_found_returns_error`
  Given: Document with node "source" but no "target"
  When: create_edge(&mut doc, NodeId::new("source"), NodeId::new("target"), edge_id)
  Then: Returns Err(RoutingError::TargetNotFound(NodeId::new("target")))

- `test_violates_p3_self_loop_returns_error`
  Given: Document with node "same"
  When: create_edge(&mut doc, NodeId::new("same"), NodeId::new("same"), edge_id)
  Then: Returns Err(RoutingError::SelfLoop(NodeId::new("same")))

- `test_violates_p4_cycle_detected_returns_error`
  Given: Document with edges A→B and B→C
  When: create_edge(&mut doc, C, A, new_edge_id)
  Then: Returns Err(RoutingError::CycleDetected)

- `test_violates_p7_invalid_nan_coordinates_returns_error`
  Given: Node with NaN x coordinate
  When: validate_edge_routing or create_edge
  Then: Returns Err(RoutingError::InvalidNodeCoordinates(node_id))

### P5-P6 Violations (Group Selection)
- `test_violates_p5_empty_selection_returns_error`
  Given: Empty editor_state.selected_items
  When: group_selection(&mut doc, &group_id)
  Then: Returns Err(GroupingError::EmptySelection)

- `test_violates_p6_locked_node_returns_error`
  Given: Selected node with locked=true
  When: group_selection(&mut doc, &group_id)
  Then: Returns Err(GroupingError::LockedNode(locked_node_id))

- `test_violates_p8_subgraph_too_small_returns_error`
  Given: validate_subgraph_bounds(0.0, 0.0, 10.0, 10.0)
  When: Function is called
  Then: Returns Err(GroupingError::SubgraphTooSmall { width: 10.0, height: 10.0 })

### P9 Violation (Ungroup Selection)
- `test_violates_p9_no_subgraph_selected_returns_error`
  Given: Selected regular nodes (not Subgraph kind)
  When: ungroup_selection(&mut doc)
  Then: Returns Err(GroupingError::EmptySelection)

## Given-When-Then Scenarios

### Scenario 1: Creating a Valid Edge
**Scenario**: User draws edge from one node to another
Given: A document with two existing nodes A and B
When: User creates an edge from A to B
Then:
- Edge is inserted into document
- Edge has correct source (A) and target (B)
- Document still satisfies DAG invariant (I1)
- Selection is not affected

### Scenario 2: Attempting Self-Loop
**Scenario**: User tries to connect node to itself
Given: A document with node A
When: User attempts to create edge A→A
Then:
- Returns Err(RoutingError::SelfLoop(A))
- No edge is created
- Document state unchanged

### Scenario 3: Creating Edge That Would Create Cycle
**Scenario**: User tries to create edge that would create a cycle
Given: Document with edges A→B and B→C
When: User attempts to create edge C→A
Then:
- Returns Err(RoutingError::CycleDetected)
- No edge is created
- Document state unchanged

### Scenario 4: Grouping Nodes into Subgraph
**Scenario**: User groups selected nodes
Given: Two nodes selected at (100,100) and (200,200) with size 50x50
When: User invokes group selection
Then:
- New subgraph node created
- Subgraph bounds include both nodes plus padding (20.0 units)
- Both nodes reparented to subgraph
- Selection updates to contain only subgraph

### Scenario 5: Grouping Locked Node Fails
**Scenario**: User tries to group locked node
Given: Selected node with locked=true
When: User invokes group selection
Then:
- Returns Err(GroupingError::LockedNode(node_id))
- No subgraph created
- Document state unchanged

### Scenario 6: Ungrouping a Subgraph
**Scenario**: User ungroups a subgraph
Given: A subgraph with three children selected
When: User invokes ungroup selection
Then:
- Subgraph node deleted from document
- All children orphaned (parent set to None or inherited)
- Children selected
- Edges connected to subgraph removed

### Scenario 7: Ungrouping Non-Subgraph Fails
**Scenario**: User tries to ungroup regular nodes
Given: Selected regular nodes (not subgraphs)
When: User invokes ungroup selection
Then:
- Returns Err(GroupingError::EmptySelection)
- Document state unchanged
