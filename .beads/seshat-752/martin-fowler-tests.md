# Martin Fowler Test Plan - seshat-752

**bead_id:** seshat-752  
**bead_title:** domain-bound: Replace String with NodeId in DomainOp  
**phase:** Test Plan  
**updated_at:** 2026-03-12T22:15:00Z

---

## DSL (Domain-Specific Language) - ATDD

This DSL defines the readable language for stakeholders:

```gherkin
Feature: NodeId Entity Identification
  Stakeholders can understand how the system validates entity identifiers

  Scenario: Valid node identifier accepted
    Given a node identifier "node-1"
    When the system processes the operation
    Then the operation succeeds
    And the identifier is stored as NodeId("node-1")

  Scenario: Empty identifier rejected
    Given an empty node identifier ""
    When the system processes the operation
    Then the operation fails with InvalidNodeId error
    And no NodeId is created

  Scenario: Whitespace-only identifier rejected
    Given a whitespace-only node identifier " "
    When the system creates the NodeId
    Then the operation fails with InvalidNodeId error
    And no NodeId is created

  Scenario: Malformed JSON rejected
    Given malformed JSON input
    When the system parses the operation
    Then the operation fails with InvalidJson error
```

---

## Test Categories

### 1. Happy Path Tests (Real Execution - cargo test)

| Test Name | Given | When | Then |
|-----------|-------|------|------|
| `given_valid_node_id_when_submit_update_label_then_nodeid_stored` | Valid NodeId string | Operation submitted | Returns `Ok(UpdateLabel { target_id: NodeId(...) })` |
| `given_valid_node_when_creating_node_then_nodeid_wraps_id` | Valid node JSON with non-empty id | Operation processed | NodeId wraps the id string |
| `given_valid_node_when_deleting_node_then_nodeid_wrapped` | Valid node id | Operation processed | Returns NodeId wrapper |
| `given_valid_source_and_target_when_adding_edge_then_edgeid_created` | Valid source/target node ids | Operation processed | Returns EdgeId wrapping both |
| `given_valid_update_label_when_serialized_then_deserialized_equals_original` | Valid UpdateLabel | Serialize → Deserialize | NodeId equals original |

### 2. Error Path Tests

| Test Name | Given | When | Then |
|-----------|-------|------|------|
| `given_empty_string_when_creating_nodeid_then_invalidnodeid_returned` | Empty string `""` | NodeId created | Returns `Err(InvalidNodeId)` |
| `given_malformed_json_when_parsing_domainop_then_invalidjson_returned` | Invalid JSON | Parse DomainOp | Returns `Err(InvalidJson)` |
| `given_unknown_op_type_when_parsing_then_unknownoptype_returned` | Unknown op_type | Parse DomainOp | Returns `Err(UnknownOpType)` |
| `given_missing_target_id_when_parsing_update_label_then_missingfield_returned` | Missing target_id | Parse UpdateLabel | Returns `Err(MissingField)` |

### 3. Edge Case Tests

| Test Name | Given | When | Then |
|-----------|-------|------|------|
| `given_whitespace_only_when_creating_nodeid_then_error_returned` | `" "` (whitespace only) | NodeId created | Returns error (whitespace rejected) |
| `given_unicode_string_when_creating_nodeid_then_success` | Unicode string `Node-日本語-123` | NodeId created | Successfully creates NodeId |
| `given_very_long_string_when_creating_nodeid_then_success` | Long string (10KB) | NodeId created | Successfully creates NodeId |

### 4. Integration Tests (Testing Trophy)

| Test Name | Description |
|-----------|-------------|
| `integration_update_label_persists_to_event_log` | Full pipeline: JSON → parse → apply → verify in event log |
| `integration_update_label_with_conflict_resolution` | Two concurrent updates to same node |
| `integration_document_apply_update_label` | Document.apply(UpdateLabel) mutates diagram state |

### 5. Property-Based Tests

| Test Name | Property |
|-----------|----------|
| `property_all_valid_nodeids_are_nonempty` | All successful NodeId creations return non-empty |
| `property_update_label_serialization_idempotent` | Serialization is idempotent |
| `property_domainop_roundtrip_preserves_target_id` | Roundtrip preserves target_id value |

### 6. Fuzzing Tests

| Test Name | Description |
|-----------|-------------|
| `fuzz_json_parsing_rejects_invalid_input` | Random malformed JSON never panics, always returns error |
| `fuzz_nodeid_creation_rejects_garbage` | Random garbage strings either reject or sanitize |

---

## BDD Scenarios (Dan North Format)

### Scenario 1: Valid Node Label Update
**Given** a user has created a node with ID "node-1"  
**When** the user submits an UpdateLabel operation with target_id="node-1"  
**Then** the system accepts the operation and stores NodeId("node-1")  

### Scenario 2: Empty ID Rejection  
**Given** a user attempts to update a node with empty target_id  
**When** the system parses the UpdateLabel  
**Then** the system returns InvalidNodeId error  

### Scenario 3: Malformed JSON Rejection
**Given** a user submits malformed JSON  
**When** the system parses the DomainOp  
**Then** the system returns InvalidJson error  

### Scenario 4: Roundtrip Preservation
**Given** a valid UpdateLabel with NodeId  
**When** the user serializes then deserializes the operation  
**Then** the resulting NodeId equals the original  

---

## DomainOp Variants Coverage

All DomainOp variants that use entity identifiers:

| Variant | ID Field | Type | Test Coverage |
|---------|----------|------|---------------|
| UpdateLabel | target_id | NodeId | ✅ |
| NodeAdd | node_id | NodeId | ✅ |
| NodeDelete | node_id | NodeId | ✅ |
| EdgeAdd | source_id, target_id | NodeId | ✅ |
| EdgeDelete | edge_id | EdgeId | Not in scope (EdgeId exists) |
| EdgeRelabel | edge_id | EdgeId | Not in scope |

---

## Contract Verification Tests

| Contract Item | Test Verifying It |
|---------------|-------------------|
| P1: Valid JSON | `given_malformed_json_when_parsing_domainop_then_invalidjson_returned` |
| P2: Non-empty ID | `given_empty_string_when_creating_nodeid_then_invalidnodeid_returned` |
| P3: Known op_type | `given_unknown_op_type_when_parsing_then_unknownoptype_returned` |
| Q1: NodeId types | `given_valid_node_id_when_submit_update_label_then_nodeid_stored` |
| Q2: Valid wrappers | `property_all_valid_nodeids_are_nonempty` |
| Q3: Roundtrip | `given_valid_update_label_when_serialized_then_deserialized_equals_original` |
| I1: No naked String | Code inspection + behavior tests |
| I2: Non-empty | `given_empty_string_when_creating_nodeid_then_invalidnodeid_returned` |
| I3: Errors not panics | All error path tests |

---

## Test Execution

All tests run with:
```bash
cargo test
```

No `#[cfg(kani)]` gates - tests run in real execution environment.
