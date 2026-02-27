# Martin Fowler Test Plan

## Test Strategy

Tests for this feature are primarily **integration/e2e** tests since the behavior involves:
1. JavaScript execution in browser context
2. Event listener lifecycle management
3. Dioxus effect reactivity

Unit tests in pure Rust cannot verify JavaScript-side behavior. Tests below assume WASM target with browser environment (e.g., `wasm-bindgen-test` or Playwright).

## Happy Path Tests

### test_effect_registers_keyboard_listener
- **Given**: Component with `use_global_keyboard()` is mounted
- **When**: Effect runs to completion
- **Then**: `window.__seshat_global_keyboard_cleanup` is defined as a function

### test_cleanup_removes_listener
- **Given**: Effect has run and listener is registered
- **When**: `window.__seshat_global_keyboard_cleanup()` is called
- **Then**: Pressing Ctrl+Z does **not** trigger undo action

### test_effect_re_run_removes_old_listener
- **Given**: Effect has run once
- **When**: Effect re-runs (due to reactive dependency change)
- **Then**: Only one listener is active (no duplicates)

### test_keyboard_shortcuts_fire_after_effect
- **Given**: Component with `use_global_keyboard()` is mounted
- **When**: User presses Ctrl+Z (not in input field)
- **Then**: `apply_undo` is called with correct signals

## Error Path Tests

### test_context_missing_diagram_document_panics
- **Given**: Component tree without `Signal<DiagramDocument>` context
- **When**: `use_global_keyboard()` is called
- **Then**: Panics with context not found error

### test_context_missing_history_panics
- **Given**: Component tree without `Signal<History>` context
- **When**: `use_global_keyboard()` is called
- **Then**: Panics with context not found error

### test_ignored_when_input_focused
- **Given**: Effect has run, an `<input>` element is focused
- **When**: User presses Ctrl+Z
- **Then**: `apply_undo` is **not** called (event not handled)

### test_ignored_when_textarea_focused
- **Given**: Effect has run, a `<textarea>` element is focused
- **When**: User presses Ctrl+Z
- **Then**: `apply_undo` is **not** called

### test_ignored_when_contenteditable_focused
- **Given**: Effect has run, a `contenteditable` element is focused
- **When**: User presses Ctrl+Z
- **Then**: `apply_undo` is **not** called

## Edge Case Tests

### test_cleanup_is_idempotent
- **Given**: Effect has run and listener is registered
- **When**: `window.__seshat_global_keyboard_cleanup()` is called **twice**
- **Then**: No error is thrown, second call is a no-op

### test_multiple_effect_runs_no_listener_accumulation
- **Given**: Effect has run 5 times (simulating reactive updates)
- **When**: User presses Ctrl+Z once
- **Then**: `apply_undo` is called exactly **once** (not 5 times)

### test_eval_channel_closed_gracefully
- **Given**: Effect has run, eval channel is closed externally
- **When**: `eval.recv()` returns error
- **Then**: Spawned task terminates without panic

## Contract Verification Tests

### test_precondition_p1_diagram_document_context
- **Verifies**: P1 (DiagramDocument context must exist)
- **Given**: No `Signal<DiagramDocument>` in context
- **When**: `use_global_keyboard()` called
- **Then**: Panic occurs (compile-time enforceable via type system)

### test_precondition_p2_history_context
- **Verifies**: P2 (History context must exist)
- **Given**: No `Signal<History>` in context
- **When**: `use_global_keyboard()` called
- **Then**: Panic occurs

### test_postcondition_q1_single_listener
- **Verifies**: Q1 (exactly one listener after effect)
- **Given**: Fresh component mount
- **When**: Effect completes
- **Then**: `window.getEventListenerCount(window, 'keydown') === 1` (or equivalent check)

### test_postcondition_q2_cleanup_function_exists
- **Verifies**: Q2 (cleanup function exists)
- **Given**: Effect has run
- **When**: Check `typeof window.__seshat_global_keyboard_cleanup`
- **Then**: Returns `'function'`

### test_postcondition_q3_cleanup_removes_listener
- **Verifies**: Q3 (cleanup removes listener)
- **Given**: Effect has run, listener active
- **When**: Call cleanup, then check listener count
- **Then**: Listener count is 0

### test_postcondition_q4_no_accumulation_on_rerun
- **Verifies**: Q4 (previous listener removed before new one)
- **Given**: Effect has run once
- **When**: Trigger effect re-run, check listener count
- **Then**: Count is still 1 (not 2)

### test_invariant_i1_single_listener_at_any_time
- **Verifies**: I1 (no more than one listener active)
- **Given**: Multiple rapid effect re-runs
- **When**: Check listener count at any point
- **Then**: Count is always ≤ 1

## Contract Violation Tests

### test_p1_violation_returns_panic
**Matches violation**: VIOLATES P1
```rust
#[test]
#[should_panic(expected = "use_context")]
fn test_p1_violation_returns_panic() {
    // Given: No DiagramDocument context provider
    // When: use_global_keyboard() is called
    // Then: Panics with context error
}
```

### test_p2_violation_returns_panic
**Matches violation**: VIOLATES P2
```rust
#[test]
#[should_panic(expected = "use_context")]
fn test_p2_violation_returns_panic() {
    // Given: No History context provider
    // When: use_global_keyboard() is called
    // Then: Panics with context error
}
```

### test_q1_violation_duplicate_listeners
**Matches violation**: VIOLATES Q1 (detects the bug this bead fixes)
```rust
#[wasm_bindgen_test]
fn test_q1_violation_duplicate_listeners() {
    // Given: Effect runs without cleanup (current bug)
    // When: Effect re-runs 3 times
    // Then: Listener count is 3 (BUG - should be 1)
    // After fix: Listener count is 1
}
```

### test_q2_violation_cleanup_undefined
**Matches violation**: VIOLATES Q2
```javascript
// e2e test (Playwright/Cypress)
test('q2_violation_cleanup_undefined', async () => {
    // Given: Effect runs without setting cleanup function
    // When: Check window.__seshat_global_keyboard_cleanup
    // Then: undefined (BUG - should be function)
});
```

### test_q3_violation_cleanup_does_not_remove
**Matches violation**: VIOLATES Q3
```javascript
test('q3_violation_cleanup_does_not_remove', async () => {
    // Given: Cleanup function exists but doesn't remove listener
    // When: Call cleanup, press Ctrl+Z
    // Then: Undo still fires (BUG - should not fire)
});
```

### test_q4_violation_accumulation_on_rerun
**Matches violation**: VIOLATES Q4
```rust
#[wasm_bindgen_test]
fn test_q4_violation_accumulation_on_rerun() {
    // Given: Effect re-runs without calling prior cleanup
    // When: Trigger reactive update causing re-run
    // Then: Undo fires multiple times on single Ctrl+Z (BUG)
}
```

## Given-When-Then Scenarios

### Scenario 1: Normal Operation
**Given** the application has loaded with `use_global_keyboard()` active
**When** the user presses Ctrl+Z outside of any input field
**Then** the undo action is triggered exactly once

### Scenario 2: Effect Re-run (Cleanup Required)
**Given** the effect has already registered a listener
**When** a reactive dependency changes causing the effect to re-run
**Then** the old listener is removed before the new one is added
**And** only one listener exists at any time

### Scenario 3: Component Unmount
**Given** a component using `use_global_keyboard()` is mounted
**When** the component is unmounted (e.g., navigating away)
**Then** the cleanup function should be available for manual cleanup
**Note**: Dioxus `use_effect` cleanup is automatic when scope drops

### Scenario 4: Input Field Interaction
**Given** an `<input>` element has focus
**When** the user presses Ctrl+Z
**Then** the browser's native undo is used (event not captured)
**And** `apply_undo` is not called

## Test Implementation Notes

1. **Rust unit tests**: Can verify context panics and type signatures
2. **WASM tests**: Use `wasm-bindgen-test` for effect behavior
3. **E2E tests**: Use Playwright for full keyboard interaction testing
4. **Manual verification**: Use browser DevTools `getEventListeners(window)` to verify listener count

## Exit Criteria Checklist

- [x] Every precondition has a type encoding specified
- [x] Every precondition and postcondition has a concrete violation example
- [x] Every violation example has a matching named test
- [x] Every `&mut` parameter has mutation postconditions listed (N/A - no mutable borrows)
- [x] Every failure mode has a corresponding error variant (panic/channel error)
- [x] Test names describe behavior unambiguously
