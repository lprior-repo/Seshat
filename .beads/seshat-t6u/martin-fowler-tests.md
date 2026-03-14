# Martin Fowler Test Plan: Subgraph Creation (SUB-003 to SUB-007)

## Happy Path Tests (SUB-003)
- **test_sub003_creates_subgraph_node_from_selected_nodes**
  - Given: A document with three nodes at root level selected.
  - When: `group_selection` is called.
  - Then: A new `NodeKind::Subgraph` is created with unique ID.
- **test_sub003_reparents_selected_nodes_to_new_subgraph**
  - Given: Nodes A, B, and C selected at root level.
  - When: `group_selection(doc, "G1")` is called.
  - Then: A.parent, B.parent, and C.parent are all `Some("G1")`.
- **test_sub003_mixed_parent_grouping_reparents_to_common_ancestor**
  - Given:
    - Node A (parent: "S1")
    - Node B (parent: "S1")
    - Node C (parent: "S2" where S2.parent == "S1")
    - Selection = {A, B, C}
  - When: `group_selection(doc, "G1")` is called.
  - Then:
    - G1.parent == "S1" (the LCA of A, B, C).
    - A, B, C all have parent "G1".
- **test_sub003_boundary_success_nesting_at_depth_4**
  - Given: A node `A` at depth 4 (e.g., S1 -> S2 -> S3 -> S4 -> A).
  - When: `group_selection(doc, "G1")` called on {A}.
  - Then: `G1` created at depth 4, `A` moves to depth 5 (max allowed). Result is SUCCESS.

## Happy Path Tests (SUB-004)
- **test_sub004_subgraph_bounds_encompass_all_children_plus_padding**
  - Given: Node A at (10, 10, 100x50) and Node B at (150, 200, 50x100).
  - When: `group_selection` is called with padding 24.0.
  - Then: Subgraph bounds encompass child min/max: x_range=[10, 200], y_range=[10, 300]. With padding, it is (-14, -14, 238, 338).

## Happy Path Tests (SUB-005)
- **test_sub005_selection_replaces_children_with_new_subgraph**
  - Given: Nodes A and B selected.
  - When: `group_selection` is called.
  - Then: `doc.editor_state.selected_items` contains ONLY the new Subgraph's ID.

## Happy Path Tests (SUB-006)
- **test_sub006_z_index_is_min_of_children_minus_one**
  - Given: Node A (z=100) and Node B (z=50) selected.
  - When: `group_selection` is called.
  - Then: The new `Subgraph` has `z_index = 49`.

## Happy Path Tests (SUB-007)
- **test_sub007_atomicity_and_undo**
  - Given: Nodes A and B selected.
  - When: `group_selection` is called, followed by `doc.undo()`.
  - Then: The Subgraph is removed, A and B have their previous `parent`, and original Selection is restored.
- **test_sub007_wal_persistence_consistency**
  - Given: An empty WAL.
  - When: `group_selection` is called.
  - Then: Exactly one `EventEnvelope` with `operation: DomainOp::Group` is persisted.

## Invariant Verification Tests
- **test_invariant_i1_no_orphaned_children**
  - Given: Successful grouping into `G1`.
  - When: Verified.
  - Then: ∀ child_id where doc.nodes[child_id].parent == "G1", doc.nodes.contains(child_id).
- **test_invariant_i2_no_circular_parents**
  - Given: Successful grouping of `A` and `B` into `G1`.
  - When: `check_nesting_depth` is called.
  - Then: Parent chain traversal for `G1` reaches `None` in ≤ 5 steps.

## Error Path Tests
- **test_err_empty_selection_returns_error**
  - Given: An empty selection.
  - When: `group_selection` is called.
  - Then: Returns `Err(GroupingError::EmptySelection)`.
- **test_err_locked_node_returns_all_locked_ids**
  - Given: Node A (locked) and Node B (locked) in selection.
  - When: `group_selection` is called.
  - Then: Returns `Err(GroupingError::LockedNode(vec![A_id, B_id]))`.
- **test_err_nesting_depth_exceeded_returns_error**
  - Given: A node at depth 5 selected.
  - When: `group_selection` is called (which would move it to depth 6).
  - Then: Returns `Err(GroupingError::NestedSubgraphLimitExceeded(5))`.

## E2E UI Smoke Test
- **test_ui_selection_sync_after_grouping**
  - Given: A Dioxus mock environment with `DiagramDocument` and two nodes selected in the UI.
  - When: Group shortcut (Ctrl+G) is triggered.
  - Then: The visual selection rectangle updates to surround the new Subgraph, and child nodes no longer show selection handles.

## Property-Based Tests (Proptest)
- **test_proptest_subgraph_bounds_always_encompass_children**
  - Strategy: Generate random sets of N nodes (1-20) with random positions and sizes.
  - Invariant: ∀ node ∈ children, node.bounds ⊆ (subgraph.bounds - padding).

## Contract Violation Tests
- **test_v1_empty_selection_violation**
  - Given: Document with 10 nodes, but 0 selected.
  - When: `group_selection` called.
  - Then: Returns `Err(GroupingError::EmptySelection)`.
- **test_v2_node_not_found_violation**
  - Given: Selection set containing `{"non-existent-uuid"}`.
  - When: `group_selection` called.
  - Then: Returns `Err(GroupingError::NodeNotFound("non-existent-uuid"))`.
- **test_v3_locked_node_violation**
  - Given: Nodes `B` and `C` both having `locked: true`.
  - When: `group_selection` called.
  - Then: Returns `Err(GroupingError::LockedNode(vec!["B", "C"]))`.
- **test_v4_invalid_coordinates_violation**
  - Given: Node `C` with `x: f64::NAN`.
  - When: `group_selection` called.
  - Then: Returns `Err(GroupingError::InvalidCoordinates)`.
- **test_v5_depth_violation**
  - Given: A node at depth 5 selected (moving it to 6 is forbidden).
  - When: `group_selection` called.
  - Then: Returns `Err(GroupingError::NestedSubgraphLimitExceeded(5))`.

## Given-When-Then Scenarios
### Scenario 1: Grouping with disparate z-indices
Given:
- Node A (0, 0, 10, 10), `z_index: 200`
- Node B (20, 20, 10, 10), `z_index: 50`
- Selection = {A, B}
When:
- `group_selection` is called with `group_id="G1"`
Then:
- G1.z_index == 49
- A.parent == "G1", B.parent == "G1"
- Selection = {"G1"}
