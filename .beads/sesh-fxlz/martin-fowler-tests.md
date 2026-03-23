# Martin Fowler Test Plan

## Domain Contract Tests (Core Logic)
- `test_apply_edge_label_edit_updates_label_on_success`: Verifies `apply_edge_label_edit` updates the edge's label text when the edge exists.
- `test_apply_edge_label_edit_returns_error_for_missing_edge`: Verifies `apply_edge_label_edit` returns an error and does not mutate the document when `edge_id` is invalid.
- `test_apply_edge_label_edit_returns_error_on_persistence_failure`: Verifies `apply_edge_label_edit` handles underlying storage failures gracefully.
- `test_apply_edge_label_edit_accepts_empty_string`: Verifies the domain allows empty string labels.
- `test_apply_edge_label_edit_accepts_very_long_string`: Verifies the domain handles maximum length constraints gracefully.

## Integration Tests (Testing Trophy)
- `test_integration_commit_edge_label_updates_domain_document_without_mocks`: True integration test that drives the UI presentation layer to commit an edge label edit, and verifies the actual underlying domain document state is updated, utilizing the real domain logic.
- `test_integration_cancel_edge_label_leaves_domain_document_unchanged_without_mocks`: True integration test that drives the UI presentation layer to cancel an edge label edit, verifying the real underlying domain document remains unchanged.

## Contract Verification Tests
- `test_postcondition_edge_label_matches_new_text`: Property-based test ensuring the edge label always matches the provided string on success.
- `test_invariant_edge_connectivity_unchanged`: Property-based test ensuring source and target nodes are never modified by a label edit.

## Given-When-Then Scenarios (ATDD / BDD)

### Scenario 1: Committing edge text (Happy Path)
Given: An edge exists in the domain document
When: The user commits the drafted label "New Connection" for the edge
Then:
- The edge in the domain document has the label "New Connection"

### Scenario 2: Canceling edge text
Given: An edge exists in the domain document with the label "Old Label"
When: The user drafts "Discarded Label" and cancels the edit
Then:
- The edge in the domain document retains the label "Old Label"

### Scenario 3: Committing edge text for a non-existent edge (Error Path)
Given: An edge has been deleted from the domain document
When: The user attempts to commit an edit for the deleted edge
Then:
- The domain document remains unchanged
- The system returns a TargetNotFound error

### Scenario 4: Committing edge text when saving fails (Error Path)
Given: The underlying storage system is in a failed state
When: The user commits an edit for an edge
Then:
- The domain document remains unchanged
- The system returns an UpdateFailed error

### Scenario 5: Committing empty edge text (Edge Case)
Given: An edge exists in the domain document
When: The user commits an empty string label for the edge
Then:
- The edge in the domain document has an empty label

### Scenario 6: Committing very long edge text (Edge Case)
Given: An edge exists in the domain document
When: The user commits a 10,000 character string label for the edge
Then:
- The edge in the domain document has the 10,000 character string label