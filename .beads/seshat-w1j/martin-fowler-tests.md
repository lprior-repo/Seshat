# Martin Fowler Test Plan - Bead seshat-w1j

## Metadata
- bead_id: seshat-w1j
- bead_title: UI Dispatch: Edge Connection
- phase: TEST_PLAN
- updated_at: 2026-03-12T12:00:00Z

## Overview
This test plan specifies the behavior of `handle_edge_drawing_complete` which wires the toolbar Edge Connect button to construct `DomainOp::EdgeConnect` and dispatch to db_tx.

## Happy Path Tests

### test_returns_success_when_valid_nodes_provided
Given: A DiagramDocument with two existing nodes ("node-1" and "node-2"), db_tx is Some channel
When: handle_edge_drawing_complete(db_tx, &doc, "node-1", "node-2")
Then: Returns Ok(DispatchResult) with nodes_affected >= 1 and dispatches_sent >= 1

### test_dispatches_edge_connect_envelope
Given: A DiagramDocument with two existing nodes, db_tx is Some(channel)
When: handle_edge_drawing_complete(db_tx, &doc, "node-1", "node-2")
Then: Channel receives an EventEnvelope with operation DomainOp::EdgeConnect

### test_generates_unique_edge_id
Given: A DiagramDocument with two existing nodes
When: handle_edge_drawing_complete is called twice with same source/target
Then: Each call generates a unique edge ID (UUID)

## Error Path Tests

### test_returns_edge_not_found_when_source_empty
Given: A DiagramDocument with existing node "node-2"
When: handle_edge_drawing_complete(db_tx, &doc, "", "node-2")
Then: Returns Err(DispatchError::EdgeNotFound)

### test_returns_edge_not_found_when_target_empty
Given: A DiagramDocument with existing node "node-1"
When: handle_edge_drawing_complete(db_tx, &doc, "node-1", "")
Then: Returns Err(DispatchError::EdgeNotFound)

### test_returns_edge_not_found_when_source_not_in_document
Given: A DiagramDocument with existing node "node-1"
When: handle_edge_drawing_complete(db_tx, &doc, "nonexistent", "node-1")
Then: Returns Err(DispatchError::EdgeNotFound)

### test_returns_edge_not_found_when_target_not_in_document
Given: A DiagramDocument with existing node "node-1"
When: handle_edge_drawing_complete(db_tx, &doc, "node-1", "nonexistent")
Then: Returns Err(DispatchError::EdgeNotFound)

### test_returns_self_loop_error_when_source_equals_target
Given: A DiagramDocument with existing node "node-1"
When: handle_edge_drawing_complete(db_tx, &doc, "node-1", "node-1")
Then: Returns Err(DispatchError::SelfLoop)

### test_returns_cycle_detected_when_edge_creates_cycle
Given: A DiagramDocument with existing edge "node-2" -> "node-3" (creates cycle if reversed)
When: handle_edge_drawing_complete(db_tx, &doc, "node-3", "node-2")
Then: Returns Err(DispatchError::CycleDetected)

### test_returns_channel_missing_when_db_tx_none
Given: A DiagramDocument with two existing nodes, db_tx is None
When: handle_edge_drawing_complete(None, &doc, "node-1", "node-2")
Then: Returns Err(DispatchError::ChannelMissing)

## Edge Case Tests

### test_handles_single_node_document
Given: A DiagramDocument with exactly one node
When: handle_edge_drawing_complete is called
Then: Returns EdgeNotFound (target doesn't exist)

### test_handles_document_with_no_nodes
Given: A DiagramDocument with no nodes
When: handle_edge_drawing_complete(db_tx, &doc, "any", "any")
Then: Returns EdgeNotFound

### test_handles_many_nodes_document
Given: A DiagramDocument with 100+ nodes
When: handle_edge_drawing_complete(db_tx, &doc, "node-1", "node-100")
Then: Returns Ok with successful dispatch

## Contract Verification Tests

### test_precondition_p1_source_non_empty
Given: Any DiagramDocument
When: handle_edge_drawing_complete(db_tx, &doc, "", "target")
Then: Returns Err(DispatchError::EdgeNotFound)

### test_precondition_p2_target_non_empty
Given: Any DiagramDocument
When: handle_edge_drawing_complete(db_tx, &doc, "source", "")
Then: Returns Err(DispatchError::EdgeNotFound)

### test_precondition_p3_source_exists
Given: DiagramDocument without "missing-source"
When: handle_edge_drawing_complete(db_tx, &doc, "missing-source", "node-2")
Then: Returns Err(DispatchError::EdgeNotFound)

### test_precondition_p4_target_exists
Given: DiagramDocument without "missing-target"
When: handle_edge_drawing_complete(db_tx, &doc, "node-1", "missing-target")
Then: Returns Err(DispatchError::EdgeNotFound)

### test_precondition_p5_not_self_loop
Given: DiagramDocument with node "node-1"
When: handle_edge_drawing_complete(db_tx, &doc, "node-1", "node-1")
Then: Returns Err(DispatchError::SelfLoop)

### test_precondition_p6_dag_preservation
Given: DiagramDocument with path "A" -> "B" -> "C"
When: handle_edge_drawing_complete(db_tx, &doc, "C", "A")
Then: Returns Err(DispatchError::CycleDetected)

### test_precondition_p7_channel_available
Given: DiagramDocument with valid nodes, db_tx is None
When: handle_edge_drawing_complete(None, &doc, "node-1", "node-2")
Then: Returns Err(DispatchError::ChannelMissing)

### test_postcondition_q1_returns_dispatch_result
Given: Valid document with nodes and db_tx is Some
When: handle_edge_drawing_complete succeeds
Then: Returns Ok(DispatchResult) with nodes_affected >= 1

### test_postcondition_q2_envelope_sent
Given: Valid document with nodes and mock channel
When: handle_edge_drawing_complete succeeds
Then: Mock channel has received exactly 1 EventEnvelope

### test_postcondition_q3_node_ids_mapped
Given: Valid document with nodes
When: handle_edge_drawing_complete(db_tx, &doc, "node-1", "node-2") succeeds
Then: Envelope contains DomainOp::EdgeConnect with correct source "node-1" and target "node-2"

## Given-When-Then Scenarios

### Scenario 1: User completes edge drawing with valid nodes
Given: A document with nodes "SourceNode" and "TargetNode" already exists
And: The WAL channel is connected (db_tx is Some)
When: User completes drawing an edge from SourceNode to TargetNode
Then: The edge is created and persisted
And: The operation returns success with dispatch count

### Scenario 2: User attempts self-loop
Given: A document with node "SoloNode"
When: User attempts to draw edge from SoloNode to SoloNode
Then: Operation fails with SelfLoop error
And: No edge is created

### Scenario 3: User attempts edge to nonexistent node
Given: A document with node "ExistingNode" but no "GhostNode"
When: User attempts to draw edge from ExistingNode to GhostNode
Then: Operation fails with EdgeNotFound error

### Scenario 4: User draws edge that creates cycle
Given: A document with edges forming A -> B -> C
When: User attempts to draw edge from C to A
Then: Operation fails with CycleDetected error

## Test File Locations
- Unit tests: `diagram_tool/src/ui/dispatch/send/edge_tests.rs`
- Integration tests: `diagram_tool/src/ui/dispatch/send/edge_integration_tests.rs`
- Property tests: `diagram_tool/src/ui/dispatch/send/edge_property_tests.rs`
- E2E tests: `diagram_tool/tests/edge_drawing_e2e_tests.rs`
- Test helpers: `diagram_tool/src/ui/dispatch/test_helpers/mod.rs`
