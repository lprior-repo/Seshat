# Martin Fowler Test Plan

> **CONSTRAINT:** To strictly adhere to the <300 lines of code per file requirement, this comprehensive test plan MUST be physically split across the multiple modules defined below.

## File & Module Structure

### 1. `test_utils/interaction_dsl.rs`
**Purpose**: Exclusively houses the `CanvasTestDsl` builder pattern to separate WHAT from HOW. No tests here.
- `CanvasTestDsl` definition and builder methods.
  - `dsl.given_state(InteractionState::Idle)`
  - `dsl.when_raw_event(RawEvent { ... })`
  - `dsl.then_expect_state(InteractionState::Selecting(..))`
  - `dsl.then_expect_error(CanvasError::...)`

### 2. `tests/interaction_happy_error_tests.rs`
**Purpose**: Happy path, error path, and edge case scenario tests.
- **Happy Paths**:
  - `given_valid_raw_event_when_parsed_then_returns_canvas_event`
  - `given_idle_state_when_mouse_down_then_transitions_to_selecting`
  - `given_idle_state_when_mouse_move_then_transitions_to_hovering`
  - `given_selecting_state_when_mouse_up_then_transitions_to_idle`
  - `given_valid_drag_state_when_delta_applied_then_updates_cumulative_offset`
- **Error Paths**:
  - `given_unknown_raw_event_when_parsed_then_returns_unparseable_error`
  - `given_idle_state_when_drag_move_event_then_returns_invalid_transition_error`
  - `given_negative_area_when_creating_selection_bounds_then_returns_invalid_bounds_error`
- **Edge Cases**:
  - `given_zero_delta_when_dragging_then_state_unchanged`
  - `given_hovering_state_when_mouse_move_event_then_returns_same_hovering_state`
  - `given_dragging_state_when_mouse_down_event_then_returns_invalid_transition_error`

### 3. `tests/interaction_combinatorial_tests.rs`
**Purpose**: Exhaustive matrices mapping every state against every event.
- `test_exhaustive_idle_transitions`
- `test_exhaustive_hovering_transitions`
- `test_exhaustive_dragging_transitions`
- `test_exhaustive_selecting_transitions`

### 4. `tests/interaction_fuzz_prop_tests.rs`
**Purpose**: Fuzzing the boundary parsers and Property-based invariants.
- `fuzz_parse_event_never_panics`: Feed randomly generated/malformed `RawEvent` payloads to `parse_event`.
- `prop_valid_event_sequence_maintains_invariants`: Fold arbitrary length sequences of valid `CanvasEvent`s through `transition` asserting no unrepresentable states occur.

### 5. `tests/interaction_workflow_tests.rs`
**Purpose**: E2E Integration Workflows driving full execution scenarios.
- **Scenario 1: Drag Workflow Integration**
  - `given_full_drag_workflow_from_raw_inputs_when_executed_then_yields_correct_final_state`
- **Scenario 2: Selection Workflow Integration**
  - `given_full_selection_workflow_from_raw_inputs_when_executed_then_yields_correct_selection_bounds`

### 6. `tests/interaction_contract_tests.rs`
**Purpose**: Contract verification and deliberate violation testing.
- **Verification**:
  - `test_precondition_parsed_boundaries`
  - `test_precondition_semantic_coordinates`
  - `test_postcondition_consumes_old_state`
- **Violations (Matching contract.md examples)**:
  - `test_P1_violation_returns_unparseable_event` (RawEvent: "unknown_click")
  - `test_P2_violation_returns_coordinate_out_of_bounds` (f64::NAN)
  - `test_P3_violation_returns_invalid_selection_bounds` (negative area)
  - `test_P4_violation_returns_compile_error` (compile test for boolean selection mode)
  - `test_Q1_violation_returns_compile_error` (compile test for mut ref consumption)
  - `test_Q2_violation_returns_coordinate_out_of_bounds` (f64::NAN delta)
  - `test_Q3_violation_returns_invalid_transition` (Idle + DragMove)

## Given-When-Then Scenarios (Detailed in workflow tests)
### Scenario 1: Full Drag Workflow Integration
Given: `CanvasTestDsl` initialized with empty domain starting in `Idle` state
When: A raw "Mouse Down" on target, "Mouse Move" by delta, and "Mouse Up" sequence is passed into the boundary API
Then:
- The internal state transitions from `Idle` -> `Dragging` -> `Dragging` -> `Idle`
- The cumulative drag coordinates match the applied deltas exactly

### Scenario 2: Full Selection Workflow Integration
Given: `CanvasTestDsl` initialized in `Idle` state
When: A raw "Mouse Down" on background, "Mouse Move" to expand bounds, and "Mouse Up" sequence is passed into the boundary API
Then:
- The internal state transitions from `Idle` -> `Selecting` -> `Selecting` -> `Idle`
- The expected `SelectionBounds` are computed and mapped accurately