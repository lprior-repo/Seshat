# Martin Fowler Test Plan: ai_conflict_state Signal

## Overview
This test plan validates the `ai_conflict_state` Signal<Option<String>> implementation for tracking AI conflict messages in the Dioxus application.

## Happy Path Tests

### test_signal_initializes_with_none_at_startup
**Given**: App is initialized with `use_context_provider`
**When**: The app component mounts
**Then**:
- The `ai_conflict_state` signal is present in context
- Reading the signal returns `None` (no initial conflict)

### test_set_conflict_message_stores_message
**Given**: `ai_conflict_state` signal exists with value `None`
**When**: `set_conflict_message("AI operation rejected: concurrent human edit")` is called
**Then**:
- Signal now contains `Some("AI operation rejected: concurrent human edit")`
- No error is returned

### test_clear_conflict_resets_to_none
**Given**: `ai_conflict_state` signal contains `Some("conflict message")`
**When**: `clear_conflict()` is called
**Then**:
- Signal now contains `None`
- No error is returned

### test_component_can_read_conflict_state
**Given**: A component requests the signal via `use_ai_conflict_state()`
**When**: Component renders
**Then**:
- The hook returns a valid `ReadOnlySignal<Option<String>>`
- Signal value is accessible via `.read()`

## Error Path Tests

### test_set_empty_message_returns_error
**Given**: `ai_conflict_state` signal exists with value `None`
**When**: `set_conflict_message("")` is called with empty string
**Then**:
- Returns `Err(ConflictError::InvalidMessage)`
- Signal value remains `None`

### test_access_signal_before_initialization_returns_error
**Given**: App has not called `use_context_provider` for `ai_conflict_state`
**When**: Component calls `use_context::<Signal<Option<String>>>()`
**Then**:
- Returns `Err(ConflictError::SignalNotFound)` or panics (implementation-dependent)

## Edge Case Tests

### test_very_long_conflict_message_handled
**Given**: `ai_conflict_state` signal exists
**When**: Setting a message with 10,000 characters
**Then**:
- Message is stored without truncation
- Signal contains the full message

### test_unicode_message_handled
**Given**: `ai_conflict_state` signal exists
**When**: Setting a message with Unicode characters
**Then**:
- Message is stored correctly
- Signal contains the expected Unicode string

### test_multiple_consecutive_conflict_messages
**Given**: Signal contains `Some("first message")`
**When**: Setting a second conflict message "second message"
**Then**:
- Signal is updated to `Some("second message")`
- Previous message is replaced (not appended)

### test_rapid_set_and_clear_operations
**Given**: Signal exists
**When**: Rapidly calling set/clear 100 times
**Then**:
- Final state is deterministic
- No race conditions or inconsistent states

## Contract Verification Tests

### test_precondition_p1_signal_initialization
**Given**: App startup
**When**: Checking `use_context_provider` initialization
**Then**: Signal starts with `Option::<String>::None`

### test_precondition_p3_message_non_empty
**Given**: Call to `set_conflict_message`
**When**: Input validation
**Then**: Empty string is rejected with `ConflictError::InvalidMessage`

### test_postcondition_q1_rejection_stores_message
**Given**: AI event rejection occurs
**When**: Handler calls set_conflict_message
**Then**: Signal contains the rejection message

### test_postcondition_q2_resolution_clears_signal
**Given**: Signal contains a conflict message
**When**: Conflict is resolved (human releases edit)
**Then**: Signal returns to `None`

### test_invariant_i1_signal_always_valid
**Given**: Any point in app lifecycle
**When**: Reading `ai_conflict_state`
**Then**: Value is either `None` or `Some(valid_string)` where valid_string is non-empty

### test_invariant_i3_no_stale_messages
**Given**: A conflict was previously set
**When**: Resolution occurs
**Then**: Signal must not retain any previous message

## Contract Violation Tests

### test_violation_p3_empty_message_rejected
**Given**: Signal initialized
**When**: `set_conflict_message("")`
**Then**: Returns `Err(ConflictError::InvalidMessage)` -- NOT a panic, NOT an unwrap failure

### test_violation_q2_cleared_signal_is_none
**Given**: Signal had `Some("conflict")`
**When**: `clear_conflict()` is called
**Then**: `signal.read()` returns `None` -- assertion passes

## Given-When-Then Scenarios

### Scenario 1: User receives AI conflict notification
**Given**: User is editing a node while AI attempts to modify the same node
**When**: AI event is rejected due to concurrent human edit
**Then**:
- The `ai_conflict_state` signal is updated with the rejection message
- UI component reading the signal displays the conflict message
- User sees: "AI operation rejected: concurrent human edit"

### Scenario 2: User resolves conflict
**Given**: User has seen a conflict message and stops editing
**When**: User releases the edit (human operation completes)
**Then**:
- The conflict state is cleared
- Signal returns to `None`
- UI no longer displays conflict notification

### Scenario 3: New conflict replaces old
**Given**: Signal contains `Some("old conflict")` from a previous conflict
**When**: A new AI event is rejected with "new conflict"
**Then**:
- Signal contains `Some("new conflict")`
- Old message is replaced, not accumulated

### Scenario 4: Component integration
**Given**: A component uses `use_ai_conflict_state()` hook
**When**: Component renders and reads signal in JSX
**Then**:
- Component re-renders when signal changes
- Display updates to show conflict message or hides when None

## Test Naming Convention
All tests follow the pattern: `test_<what_is_being_tested>`

## Execution Notes
- These tests are primarily integration tests within the Dioxus component context
- Tests requiring actual signal mutation should use `use_signal` or `use_context` within component tests
- For pure logic tests (validation, error handling), standard unit tests apply
