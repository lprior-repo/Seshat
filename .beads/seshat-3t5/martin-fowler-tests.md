# Martin Fowler Test Plan

## Happy Path Tests

### test_show_conflict_toast_displays_warning_on_valid_conflict
**Given**: Valid `AiConflictState` with reason "Human has active edit on node 'node-1'"
**When**: `show_conflict_toast(conflict_state, toast_api)` is called
**Then**:
- Returns `Ok(ToastHandle)`
- Toast has `intent == ToastIntent::Warning`
- Toast `title == "Edit Conflict"`
- Toast `detail` contains "Human has active edit"

### test_show_conflict_toast_returns_handle_for_dismiss
**Given**: Valid conflict state and available toast queue
**When**: `show_conflict_toast()` is called
**Then**:
- Returns `Ok(ToastHandle)` with valid `ToastId`
- Caller can call `.dismiss()` on handle
- `.dismiss()` returns `true`

### test_conflict_toast_auto_dismisses_after_3_seconds
**Given**: Conflict toast is displayed
**When**: 3 seconds elapse
**Then**:
- Toast is automatically dismissed
- Toast state transitions to `dismissed == true`

## Error Path Tests

### test_show_conflict_toast_returns_error_when_conflict_state_is_none
**Given**: `conflict_state = None`
**When**: `show_conflict_toast(None, toast_api)` is called
**Then**: Returns `Err(Error::NoConflictState)`

### test_show_conflict_toast_returns_error_when_queue_is_full
**Given**: Toast queue already has 1 toast (MAX_TOASTS)
**When**: `show_conflict_toast(conflict_state, toast_api)` is called
**Then**: Returns `Err(Error::QueueFull)`

### test_show_conflict_toast_uses_generic_message_when_reason_empty
**Given**: Conflict state with empty reason string
**When**: `show_conflict_toast(conflict_state, toast_api)` is called
**Then**:
- Returns `Ok(ToastHandle)`
- Toast detail contains fallback "Edit conflict"

### test_should_show_conflict_toast_returns_false_for_none
**Given**: `conflict_state = None`
**When**: `should_show_conflict_toast(None)` is called
**Then**: Returns `Ok(false)`

### test_should_show_conflict_toast_returns_true_for_valid_state
**Given**: `conflict_state = Some(AiConflictState { reason: Some(..) })`
**When**: `should_show_conflict_toast(Some(&state))` is called
**Then**: Returns `Ok(true)`

## Edge Case Tests

### test_multiple_conflict_states_only_shows_latest
**Given**: Queue has 1 existing toast from previous conflict
**When**: New conflict occurs and `show_conflict_toast()` called
**Then**:
- Oldest toast is removed (FIFO when queue full)
- New toast is displayed
- Only 1 toast visible at any time

### test_conflict_toast_detail_includes_conflicting_entities
**Given**: Conflict state with `conflicting_entities: ["node-1", "node-2"]`
**When**: `show_conflict_toast()` is called
**Then**: Toast detail includes entity names in message

### test_toast_handle_invalid_id_returns_false_on_dismiss
**Given**: ToastHandle with non-existent ID
**When**: `.dismiss()` is called
**Then**: Returns `false` (no panic)

## Contract Verification Tests

### test_precondition_p1_violation_none_conflict_state
**Given**: `conflict_state = None`
**When**: `show_conflict_toast(None, toast_api)`
**Then**: Returns `Err(Error::NoConflictState)` - NOT a panic

### test_precondition_p2_violation_queue_full
**Given**: Queue has MAX_TOASTS (1) toasts
**When**: `show_conflict_toast(conflict_state, full_queue)`
**Then**: Returns `Err(Error::QueueFull)` - NOT a panic

### test_precondition_p3_violation_empty_reason
**Given**: Conflict state with empty reason
**When**: `show_conflict_toast(state_with_empty_reason, api)`
**Then**: Returns `Err(Error::InvalidReason)` - NOT a panic

### test_postcondition_q1_toast_has_valid_id
**Given**: Valid conflict state and available queue
**When**: `show_conflict_toast()` succeeds
**Then**: Returned `ToastHandle.id` is non-zero

### test_postcondition_q2_toast_uses_warning_intent
**Given**: Valid conflict state
**When**: `show_conflict_toast()` succeeds
**Then**: Toast has `intent == ToastIntent::Warning`

### test_postcondition_q3_toast_has_correct_title
**Given**: Valid conflict state
**When**: `show_conflict_toast()` succeeds
**Then**: Toast `title == "Edit Conflict"`

### test_postcondition_q5_auto_dismiss_timing
**Given**: Conflict toast displayed
**When**: 3000ms timer expires
**Then**: Toast is automatically dismissed

### test_invariant_i1_max_toasts_enforced
**Given**: Queue at capacity
**When**: New toast added
**Then**: `queue.items.len() <= MAX_TOASTS` (1)

### test_invariant_i2_unique_toast_ids
**Given**: Multiple toast additions
**When**: Each toast gets unique ID
**Then**: All IDs are unique and monotonically increasing

## Given-When-Then Scenarios

### Scenario 1: User receives notification of concurrent edit conflict
**Given**: User is editing a diagram
**And**: AI agent attempts an operation on the same entity
**And**: System rejects AI operation due to human priority
**When**: Conflict rejection event is processed
**Then**: Toast appears with title "Edit Conflict"
**And**: Toast shows warning intent (yellow stripe)
**And**: Toast detail explains the conflict
**And**: Toast auto-dismisses after 3 seconds
**And**: No further action required from user

### Scenario 2: Rapid consecutive conflicts
**Given**: Multiple AI operations are rejected in quick succession
**When**: Each rejection triggers `show_conflict_toast()`
**Then**: Only the most recent conflict toast is visible
**And**: Previous toasts are removed from queue
**And**: User sees at most 1 conflict notification at a time

### Scenario 3: No toast for allowed AI operations
**Given**: AI operation is allowed (no conflict)
**When**: Operation is applied successfully
**Then**: No conflict toast is shown
**And**: `should_show_conflict_toast()` returns `false`
