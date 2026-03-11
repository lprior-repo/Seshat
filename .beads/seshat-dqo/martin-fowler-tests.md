# Martin Fowler Test Plan

## Happy Path Tests
- test_mul026_commit_translation_updates_all_items_and_history
- test_mul027_commit_scaling_preserves_relative_proportions
- test_mul028_commit_transform_increments_document_version
- test_mul029_commit_transform_creates_single_composite_undo_record

## Error Path Tests
- test_mul030_commit_transform_returns_error_when_item_not_found
- test_returns_error_when_document_locked
- test_returns_error_when_persistence_fails_and_rolls_back

## Edge Case Tests
- test_commit_transform_with_identity_transform_is_noop
- test_commit_transform_with_maximum_allowed_items

## Contract Verification Tests
- test_precondition_selection_must_not_be_empty
- test_precondition_transform_must_be_valid
- test_postcondition_all_items_updated
- test_postcondition_single_undo_record
- test_postcondition_atomic_failure_rollback
- test_invariant_relative_spatial_relationships_preserved
- test_invariant_no_partial_updates

## Contract Violation Tests
- `test_p3_violation_returns_item_not_found_error`
  Given: A selection containing a non-existent item ID `missing_id`
  When: `commit_transform` is called with this selection
  Then: returns `Err(Error::ItemNotFound(missing_id))` -- NOT a panic, NOT an unwrap failure

- `test_p4_violation_returns_document_locked_error`
  Given: A document that is marked as read-only/locked
  When: `commit_transform` is called on this document
  Then: returns `Err(Error::DocumentLocked)` -- NOT a panic, NOT an unwrap failure

- `test_g4_violation_returns_persistence_failed_error`
  Given: A mocked storage backend that fails on the second item update
  When: `commit_transform` is called with a selection of two items
  Then: returns `Err(Error::PersistenceFailed)`, and the first item's state is rolled back.

## Given-When-Then Scenarios
### Scenario 1: Successful Multi-Item Translation (MUL-026)
Given: A document with items A and B selected, positioned at (0,0) and (10,10)
When: A translation transform of dx=5, dy=5 is committed
Then:
- Item A is at (5,5)
- Item B is at (15,15)
- Document history has 1 new composite undo record
- Document version is incremented

### Scenario 2: Atomic Failure Rollback (MUL-030)
Given: A document with items C and D selected
When: A transform is committed, but item D fails to update (e.g. invalid state)
Then:
- The operation returns an error
- Item C remains at its original position (rollback)
- Document history is unchanged
- Document version is unchanged