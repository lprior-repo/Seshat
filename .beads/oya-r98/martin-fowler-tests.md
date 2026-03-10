# Martin Fowler Test Plan

## Bead Info
- **Bead ID**: oya-r98
- **Title**: Self-loop edges render without crash (EDG-032)

## Happy Path Tests
- `test_create_self_loop_edge_succeeds_in_allow_mode`
  - Given: A document with one node "n1"
  - When: Creating an edge from n1 to n1 with allow_self_loop=true
  - Then: Edge is created successfully, stored in document

- `test_self_loop_edge_renders_without_crash`
  - Given: A document with node "n1" and self-loop edge n1->n1
  - When: Rendering the edge on canvas
  - Then: Returns Ok with rendered edge geometry (not panic/crash)

- `test_self_loop_edge_can_be_selected`
  - Given: A document with node "n1" and self-loop edge
  - When: Clicking on the self-loop edge
  - Then: Edge is selected, not rejected

## Error Path Tests
- `test_create_self_loop_fails_in_deny_mode`
  - Given: A document with node "n1" and cycle_policy = Deny
  - When: Attempting to create edge n1->n1
  - Then: Returns Err(RoutingError::SelfLoop(n1))

- `test_create_edge_fails_with_missing_source`
  - Given: An empty document
  - When: Creating edge from non-existent node to n1
  - Then: Returns Err(RoutingError::SourceNotFound)

## Edge Case Tests
- `test_multiple_self_loops_on_same_node`
  - Given: A document with node "n1"
  - When: Creating multiple self-loop edges (if allowed)
  - Then: Handles gracefully (rejects duplicates or allows multiple)

- `test_self_loop_with_bend_points`
  - Given: A document with node "n1" and self-loop edge with bend_points
  - When: Rendering the edge
  - Then: Renders with bend points without crash

## Contract Verification Tests
- `test_precondition_p1_nodes_exist`
  - Verify: create_edge returns error when source/target don't exist

- `test_precondition_p2_deny_mode_no_self_loop`
  - Verify: create_edge returns SelfLoop error in Deny mode

- `test_postcondition_q1_self_loop_stored`
  - Verify: After creating self-loop, edge is in document.edges

- `test_postcondition_q3_rendering_no_crash`
  - Verify: render_edge with source==target doesn't panic

## Contract Violation Tests
- `test_violates_p2_self_loop_in_deny_mode_returns_self_loop_error`
  Given: Document with cycle_policy=Deny, node n1
  When: create_edge(doc, n1, n1, edge_id, false)
  Then: returns Err(RoutingError::SelfLoop(n1))

- `test_violates_q2_currently_fails_but_should_succeed`
  Given: Document with cycle_policy=Allow, node n1
  When: create_edge(doc, n1, n1, edge_id, true)
  Then: Should succeed (currently fails with SelfLoop)

## Given-When-Then Scenarios

### Scenario 1: Self-loop edge creation in graph mode
**ID**: EDG-032-happy
Given: A diagram document with a single node "LoopNode" at position (100, 100)
  And cycle_policy is set to CyclePolicy::Allow
When: User creates an edge from "LoopNode" to "LoopNode" using the edge tool
Then: The edge is created and stored in the document
  And the edge has source == target == "LoopNode"
  And subsequent validation passes without error

### Scenario 2: Self-loop edge rendering
**ID**: EDG-032-render
Given: A diagram document with node "n1" at (100, 100) with size (80, 60)
  And a self-loop edge from n1 to n1
When: The canvas renderer processes this edge
Then: The edge renders as a loop visual (e.g., small arc or curve)
  And no panic or error occurs
  And the rendered geometry is valid (has start/end points or curve control points)

### Scenario 3: Self-loop rejected in DAG mode
**ID**: EDG-032-deny
Given: A diagram document with node "n1"
  And cycle_policy is set to CyclePolicy::Deny
When: User attempts to create self-loop edge n1->n1
Then: Operation returns Err(RoutingError::SelfLoop(n1))
  And edge is NOT added to document
