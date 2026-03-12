# Martin Fowler Test Plan

## Happy Path Tests
- test_conflict_toast_displays_when_ai_event_dropped
- test_conflict_toast_contains_correct_message
- test_conflict_toast_has_warning_intent

## Error Path Tests
- test_show_conflict_toast_returns_error_when_no_conflict_state
- test_show_conflict_toast_returns_error_when_queue_full
- test_should_show_conflict_toast_returns_false_when_none

## Edge Case Tests
- test_handles_empty_conflicting_entities_list
- test_handles_very_long_conflict_message

## Contract Verification Tests
- test_precondition_p1_toast_api_available
- test_precondition_p3_conflict_detected
- test_postcondition_q1_toast_displayed
- test_postcondition_q2_auto_dismiss_after_3_seconds
- test_postcondition_q3_conflict_state_cleared_on_dismiss

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
