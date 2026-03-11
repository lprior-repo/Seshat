# Martin Fowler Test Plan

## Happy Path Tests
- test_mutate_doc_signal_applies_schema_and_semantic_validation
- test_mutate_doc_with_history_preserves_undo_chain
- test_successful_mutation_increments_revision
- test_mutation_creates_valid_document

## Error Path Tests
- test_schema_violation_returns_mutation_error
- test_semantic_violation_returns_mutation_error
- test_mutation_error_preserves_original_document_state
- test_mutation_error_does_not_increment_revision

## Edge Case Tests
- test_empty_document_mutation_succeeds
- test_concurrent_mutations_are_serialized
- test_preserve_revision_policy_keeps_revision_unchanged

## Contract Verification Tests
- test_precondition_p1_mutations_go_through_run_mutation
- test_postcondition_q1_all_mutations_validated
- test_invariant_i1_document_always_valid_after_mutation

## Contract Violation Tests
- test_with_mut_bypass_returns_error
  Given: A mutation that bypasses validation via `doc_signal.with_mut()` creates invalid state
  When: The invalid state is committed
  Then: The document becomes invalid (VIOLATES I1)
  
- test_direct_write_bypasses_validation
  Given: Code uses `doc_signal.write()` directly without validation
  When: Invalid document is written
  Then: No error is raised, invalid state persists (VIOLATES P1)

## Given-When-Then Scenarios

### Scenario 1: Valid Node Position Update
Given: A document with a valid node at position (100, 100)
When: User moves node to (200, 200) via `mutate_doc_signal`
Then:
- Schema validation passes (coordinates are finite floats)
- Semantic validation passes (no graph violations)
- Revision increments by 1
- Document is valid

### Scenario 2: Invalid Edge Reference Rejection
Given: A document with node "A" but edge references non-existent node "B"
When: Mutation attempts to add edge with invalid target
Then:
- Schema validation may pass
- Semantic validation fails with "edge-dangling" error
- Document remains unchanged
- Error is returned, not panic

### Scenario 3: Cycle Detection in Edge Mutation
Given: A document with nodes A -> B
When: Mutation adds edge B -> A creating a cycle
Then:
- Semantic validation detects cycle
- MutationError::Schema("cycle error") is returned
- Document remains unchanged (A -> B)

### Scenario 4: Parent Reference Validation
Given: A document with a regular node and a subgraph
When: Mutation sets node's parent to another non-subgraph node
Then:
- Semantic validation fails with "invalid-parent" error
- Mutation is rejected
- Original document state preserved
