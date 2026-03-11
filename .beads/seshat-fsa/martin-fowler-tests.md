# Martin Fowler Test Plan

## Overview
This test plan follows Martin Fowler's Given-When-Then style with expressive test names that describe behavior. Tests are organized into: Happy Path, Error Path, Edge Case, Contract Verification, and Contract Violation categories.

## Test Structure
- **Given**: The initial state/setup
- **When**: The action being tested
- **Then**: The expected outcome(s)

---

## Happy Path Tests

### test_conflict_toast_appears_when_ai_conflict_detected
**Given**: The app is running with ToastQueue and ai_conflict_state signals initialized
**When**: A conflict is detected and `show_conflict_toast` is called with AiConflictInfo
**Then**:
- A toast with ToastIntent::Error is created
- The toast appears in the ToastQueue
- ai_conflict_state is set to Some(conflict_info)
- The toast has the correct title and detail

### test_conflict_toast_auto_dismisses_after_3_seconds
**Given**: A conflict toast is displayed with ai_conflict_state set to Some
**When**: 3 seconds (3000ms) elapse
**Then**:
- The toast's dismissed flag is set to true
- ai_conflict_state is set to None
- The toast remains in the queue but is visually hidden

### test_manual_dismiss_clears_conflict_state
**Given**: A conflict toast is displayed with ai_conflict_state set to Some
**When**: User clicks the dismiss button before 3 seconds
**Then**:
- The toast's dismissed flag is set to true
- ai_conflict_state is set to None
- The auto-dismiss timer is cancelled (no error from cancelled timer)

### test_multiple_conflict_toasts_replaces_previous
**Given**: A conflict toast is already displayed
**When**: A new conflict is detected and show_conflict_toast is called again
**Then**:
- The previous conflict toast is replaced
- ai_conflict_state is updated with new info
- Only one conflict toast exists in the queue

---

## Error Path Tests

### test_returns_error_when_toast_queue_not_available
**Given**: The app is running without ToastQueue context provider
**When**: Attempting to call ToastApi methods
**Then**:
- Returns Err(Error::SignalNotFound)
- Does not panic

### test_returns_error_when_js_timeout_fails
**Given**: A conflict toast is displayed
**When**: The JavaScript setTimeout call fails
**Then**:
- Returns Err(Error::JsTimeoutFailure)
- The toast remains visible
- ai_conflict_state remains Some

### test_handles_toast_not_found_gracefully
**Given**: A toast ID that no longer exists
**When**: Attempting to dismiss the non-existent toast
**Then**:
- Returns false (no change)
- Does not panic
- ai_conflict_state is handled appropriately

---

## Edge Case Tests

### test_non_conflict_toasts_do_not_auto_dismiss
**Given**: A toast with ToastIntent::Info or ToastIntent::Success
**When**: 3 seconds elapse
**Then**:
- The toast is NOT automatically dismissed
- No timer is started for non-conflict toasts

### test_handles_empty_conflict_info_message
**Given**: AiConflictInfo with empty message string
**When**: show_conflict_toast is called
**Then**:
- Toast is created with empty title/detail
- Auto-dismiss timer still functions correctly

### test_handles_rapid_conflict_creation
**Given**: Multiple conflicts detected in rapid succession (<100ms apart)
**When**: show_conflict_toast is called multiple times quickly
**Then**:
- Only the latest conflict toast is displayed
- ai_conflict_state reflects the latest conflict
- No memory leaks from cancelled timers

### test_toast_queue_remains_valid_during_auto_dismiss
**Given**: A conflict toast is being auto-dismissed
**When**: Other toast operations occur simultaneously
**Then**:
- ToastQueue remains in a consistent state
- No items are corrupted
- Other toast operations work correctly

---

## Contract Verification Tests

### test_precondition_p1_toast_queue_available
**Given**: App context is properly initialized
**When**: ToastApi::from_signal is called with a valid Signal<ToastQueue>
**Then**:
- The returned ToastApi is functional
- Can create and manage toasts

### test_precondition_p2_conflict_state_available
**Given**: ai_conflict_state signal is provided in context
**When**: Reading or writing to the signal
**Then**:
- Signal operations succeed
- State is properly readable/writable

### test_postcondition_q1_auto_dismiss_timing
**Given**: A conflict toast is created
**When**: Exactly 3000ms elapses
**Then**:
- The toast is dismissed at the 3-second mark
- Timing is accurate within 100ms tolerance

### test_postcondition_q2_state_cleared_after_dismiss
**Given**: A conflict toast is dismissed (auto or manual)
**When**: After dismiss completes
**Then**:
- ai_conflict_state.read() returns None

### test_postcondition_q4_manual_dismiss_clears_state
**Given**: A conflict toast is visible with Some state
**When**: User manually dismisses the toast
**Then**:
- ai_conflict_state is immediately set to None
- Timer is cancelled

### test_invariant_i2_single_conflict_toast
**Given**: System is running
**When**: Conflict toasts are created
**Then**:
- At most one conflict toast exists at any time
- New conflict replaces old

### test_invariant_i4_state_reflects_toast_visibility
**Given**: System is running
**When**: Conflict toast state changes
**Then**:
- ai_conflict_state is Some(_) iff conflict toast is visible
- ai_conflict_state is None when no conflict toast

---

## Contract Violation Tests

### test_violation_p1_returns_signal_not_found
**Given**: No ToastQueue in context
**When**: `ToastApi::from_signal(None)` or using outside provider scope
**Then**: Returns Err(Error::SignalNotFound)

### test_violation_p2_returns_signal_not_found
**Given**: No ai_conflict_state signal
**When**: Attempting to set conflict state
**Then**: Returns Err(Error::SignalNotFound)

### test_violation_q1_toast_not_dismissed_after_timeout
**Given**: A conflict toast
**When**: Less than 3 seconds elapse (e.g., 500ms)
**Then**: Toast is NOT yet dismissed (dismissed == false)

### test_violation_q2_state_not_cleared_prematurely
**Given**: A conflict toast is visible
**When**: 1 second elapses (before auto-dismiss)
**Then**: ai_conflict_state is still Some(_)

### test_violation_q4_manual_dismiss_required_to_clear_state
**Given**: A conflict toast with Some state
**When**: Neither auto-dismiss nor manual dismiss occurs
**Then**: ai_conflict_state remains Some(_) - state only clears on dismiss

---

## Integration Test

### test_end_to_end_conflict_toast_lifecycle
**Given**: Fresh app initialization with all signals available
**When**:
1. Conflict is detected → show_conflict_toast called
2. Wait 1 second → verify toast visible, state Some
3. Wait 2 more seconds → verify auto-dismiss triggered
4. Verify state cleared
**Then**:
- Complete lifecycle works correctly
- State transitions: None → Some → None
- Toast visibility transitions: visible → dismissed

---

## Implementation Phases

### Phase 1: Signal Setup
1. Add `AiConflictInfo` struct to appropriate module
2. Add `ai_conflict_state` Signal provider in App component
3. Verify signals are accessible in Toast context

### Phase 2: Toast API Extension
1. Add `show_conflict_toast` function
2. Integrate with existing ToastApi
3. Ensure proper error handling

### Phase 3: Auto-Dismiss Timer
1. Add `CONFLICT_TOAST_AUTO_DISMISS_MS` constant (3000)
2. Implement timer logic using document::eval + setTimeout
3. Wire timer to dismiss + clear state on completion

### Phase 4: Manual Dismiss Integration
1. Ensure manual dismiss also clears ai_conflict_state
2. Verify timer cancellation on manual dismiss

### Phase 5: Testing & Validation
1. Run happy path tests
2. Run contract verification tests
3. Verify no regressions in existing toast functionality
