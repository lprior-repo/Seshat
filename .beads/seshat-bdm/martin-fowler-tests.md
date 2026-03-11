# Martin Fowler Test Plan

## Happy Path Tests
- test_edge_connects_to_top_port_anchor_successfully
- test_edge_connects_to_custom_port_anchor_successfully
- test_edge_falls_back_to_dynamic_routing_when_no_port_specified

## Error Path Tests
- test_returns_error_when_custom_port_offset_is_out_of_bounds
- test_returns_error_when_setting_port_for_nonexistent_edge
- test_returns_error_when_node_not_found_for_port_computation

## Edge Case Tests
- test_edge_port_anchor_computes_correctly_for_zero_width_node
- test_custom_port_anchor_at_exact_boundaries_zero_and_one

## Contract Verification Tests
- test_precondition_custom_port_offset_must_be_normalized
- test_postcondition_setting_port_updates_edge_state
- test_postcondition_moving_node_updates_edge_port_absolute_position
- test_postcondition_edge_port_anchors_serialize_and_deserialize

## Contract Violation Tests

- `test_p2_violation_returns_invalid_port_offset`
  Given: `NormalizedOffset::new(OrderedFloat(1.5), OrderedFloat(0.5))`
  When: constructor is called with out of bounds `x`
  Then: returns `Err(Error::InvalidPortOffset)`

- `test_p3_violation_returns_node_not_found`
  Given: `document.set_edge_source_port(&EdgeId("edge_1"), Some(PortAnchor::Top))`
  When: the source node for "edge_1" has been removed from the document
  Then: returns `Err(Error::NodeNotFound)`

## Given-When-Then Scenarios

### Scenario 1: EDG-001 Edge connects to specific port anchors
Given: A document with node A at (0,0) with size 100x100
When: An edge is created from node A to node B, with `source_port: Some(PortAnchor::Top)`
Then:
- The edge's `source_port` is successfully stored
- The computed absolute start point for the edge is exactly (50, 0)

### Scenario 2: EDG-002 Edge defaults to dynamic routing if no port specified
Given: A document with node A and node B
When: An edge is created between them with `source_port: None` and `target_port: None`
Then:
- The edge routes dynamically based on bounding box intersection
- The computed absolute start point uses the default geometry routing

### Scenario 3: EDG-003 Edge routing updates correctly when node moves
Given: An edge bound to node A's `Bottom` port
When: Node A is translated by dx=20, dy=30
Then:
- The edge's `source_port` remains `Bottom`
- The computed absolute start point translates by exactly dx=20, dy=30

### Scenario 4: EDG-004 Edge port anchors are preserved during serialization
Given: An edge with `source_port: Custom(0.25, 0.75)` and `target_port: Right`
When: The document is serialized to JSON and then deserialized
Then:
- The deserialized edge retains `source_port: Custom(0.25, 0.75)`
- The deserialized edge retains `target_port: Right`

### Scenario 5: EDG-005 Edge rejects invalid port specifications
Given: A user or API tries to set a custom port
When: The provided offset is `(-0.1, 1.2)`
Then:
- The system returns `Err(Error::InvalidPortOffset)`
- The edge state is unmodified