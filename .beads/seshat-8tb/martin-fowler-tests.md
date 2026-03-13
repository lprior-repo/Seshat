# Martin Fowler Test Plan

## Happy Path Tests (Given-When-Then Naming)
- test_given_ai_event_dropped_when_conflict_detected_then_conflict_toast_displays
- test_given_conflict_detected_when_toast_created_then_toast_contains_correct_message
- test_given_conflict_toast_created_when_displayed_then_toast_has_warning_intent

## Error Path Tests (Given-When-Then Naming)
- test_given_no_conflict_state_in_context_when_show_conflict_toast_called_then_returns_signal_not_found_error
- test_given_toast_queue_at_capacity_when_show_conflict_toast_called_then_returns_queue_full_error
- test_given_no_conflict_state_when_should_show_conflict_toast_called_then_returns_false
- test_given_empty_conflict_reason_when_show_conflict_toast_called_then_returns_invalid_reason_error

## Edge Case Tests
- test_given_empty_conflicting_entities_list_when_conflict_detected_then_toast_displays_without_entity_names
- test_given_very_long_conflict_message_when_toast_created_then_message_truncated_gracefully

## Contract Verification Tests

### Precondition Tests
- test_precondition_p1_toast_api_available
- test_precondition_p2_ai_conflict_state_initialized
- test_precondition_p3_conflict_detected

### Postcondition Tests
- test_postcondition_q1_toast_displayed
- test_postcondition_q2_auto_dismiss_after_3_seconds
- test_postcondition_q3_conflict_state_cleared_on_dismiss
- test_postcondition_q4_manual_dismiss_clears_conflict_state

### Invariant Tests
- test_invariant_i1_only_one_toast_at_a_time
- test_invariant_i2_conflict_state_reflects_conflict_existence

## Contract Violation Tests
- test_given_no_toast_api_in_context_when_show_conflict_toast_called_then_returns_signal_not_found_error
  - Given: No ToastApi signal available in Dioxus context
  - When: show_conflict_toast is called
  - Then: returns Err(Error::SignalNotFound)

- test_given_empty_ai_conflict_state_when_show_conflict_toast_called_then_returns_no_conflict_state_error
  - Given: AiConflictState with empty reason and no conflicting entities
  - When: show_conflict_toast is called
  - Then: returns Err(Error::NoConflictState)

- test_given_empty_conflict_reason_when_show_conflict_toast_called_then_returns_invalid_reason_error
  - Given: AiConflictState with empty reason string
  - When: show_conflict_toast is called
  - Then: returns Err(Error::InvalidReason)

## Property-Based Testing Considerations
- test_property_conflict_state_serializes_and_deserializes_correctly
- test_property_multiple_conflict_states_maintain_identity
- test_property_conflict_state_cloning_preserves_data
- Note: Consider using proptest or quickcheck for generative testing of AiConflictState fields:
  - reason: non-empty strings up to 500 chars
  - conflicting_entities: vec of entity IDs (0-100 items)
  - timestamp: valid DateTime values

## Integration / E2E Tests

### test_integration_poller_to_toast_pipeline
- Integration test verifying the full pipeline:
  - Given: WAL contains human edit at op_id=5
  - Given: AI event arrives for op_id=5
  - When: Poller detects dropped AI event
  - When: detect_dropped_ai_events returns has_conflict=true
  - Then: ai_conflict_state is set to Some(AiConflictState)
  - Then: show_conflict_toast is invoked
  - Then: Toast with title "Edit Conflict" appears in UI

### test_integration_manual_dismiss_clears_state_end_to_end
- E2E test for manual dismiss workflow:
  - Given: Conflict toast is displayed with ai_conflict_state=Some(...)
  - When: User clicks dismiss button on toast
  - Then: Toast is removed from queue
  - Then: ai_conflict_state becomes None
  - Then: No more toasts remain for this conflict

### test_integration_rapid_conflicts_only_show_one_toast
- E2E test for rapid conflict detection:
  - Given: First conflict toast is displayed
  - When: Second conflict detected 100ms later
  - Then: Only first toast remains visible
  - Then: New conflict is queued but not displayed until first dismissed

## Given-When-Then Scenarios

### Scenario 1: AI operation rejected due to human priority
Given: Poller detects dropped AI operation (op_id not in WAL)
When: detect_dropped_ai_events returns has_conflict=true
Then:
- ai_conflict_state is set to Some(AiConflictState)
- show_conflict_toast is called
- Toast with title "Edit Conflict" appears
- Toast auto-dismisses after 3 seconds

### Scenario 2: User manually dismisses conflict toast
Given: Conflict toast is displayed
When: User clicks dismiss button
Then:
- Toast is removed from queue
- ai_conflict_state is set to None
- No more toasts remain for this conflict

### Scenario 3: Multiple conflicts in rapid succession
Given: First conflict toast is displayed
When: Second conflict detected while first toast visible
Then:
- Only first toast remains (MAX_TOASTS = 1)
- New conflict message is not displayed until first is dismissed
- This prevents toast spam

### Scenario 4: Conflict state resolved via auto-dismiss
Given: Conflict toast displayed with Warning intent
When: 3 seconds pass (CONFLICT_TOAST_DISMISS_MS)
Then:
- Toast is automatically dismissed
- ai_conflict_state is cleared to None
- User can continue editing
