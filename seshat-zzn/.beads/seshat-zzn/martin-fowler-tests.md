# Martin Fowler Test Plan: SEL-005 to SEL-009

## Overview
This test plan specifies unit tests for selection interaction behaviors:
- SEL-005: Marquee direction switches mode (already covered in seshat-9n1)
- SEL-006: Hover shows visual affordances
- SEL-007: Resize handles are clickable
- SEL-008: Touch has larger hit area (WCAG 44px)
- SEL-009: Drag threshold prevents accidental drag

## Test Structure

Tests should be placed in: `diagram_tool/src/models/selection_ops_tests.rs`

---

## Happy Path Tests

### SEL-006: Hover Shows Visual Affordances

**test_sel_006_hover_state_changes_on_mouseenter**
- Given: A document with Node A at position (0, 0) with size 100x100
- When: Mouse enters Node A's bounds
- Then: Hover state transitions to Hovering(NodeId("n1"))

**test_sel_006_hover_state_clears_on_mouseleave**
- Given: Hover state is Hovering(NodeId("n1"))
- When: Mouse leaves Node A's bounds
- Then: Hover state transitions to Idle

**test_sel_006_hover_state_persists_during_mousemove**
- Given: Hover state is Hovering(NodeId("n1"))
- When: Mouse moves within Node A's bounds
- Then: Hover state remains Hovering(NodeId("n1"))

### SEL-007: Resize Handles Are Clickable

**test_sel_007_single_node_selection_has_eight_handles**
- Given: A document with Node A selected
- When: Selection bounds are computed for Node A
- Then: 8 handles are returned (NW, N, NE, E, SE, S, SW, W)

**test_sel_007_multi_node_selection_has_eight_handles**
- Given: A document with Node A and Node B selected (non-overlapping)
- When: Selection bounds are computed
- Then: 8 handles are returned covering the union bounds

**test_sel_007_handle_hit_test_returns_correct_handle**
- Given: Selection bounds with 8 handles
- When: Testing hit at handle position (e.g., NW corner)
- Then: Returns the correct handle (NW)

**test_sel_007_handle_hit_test_returns_none_outside_handles**
- Given: Selection bounds with 8 handles
- When: Testing hit at position outside all handles
- Then: Returns None

### SEL-008: Touch Has Larger Hit Area (WCAG 44px)

**test_sel_008_touch_hit_radius_extends_to_44px_for_small_nodes**
- Given: Base hit radius of 10.0 pixels (small node)
- When: `touch_hit_radius(10.0, true)` is called (touch input)
- Then: Returns max(10.0, 44.0) = 44.0 (WCAG minimum touch target)

**test_sel_008_touch_hit_radius_uses_base_for_large_nodes**
- Given: Base hit radius of 50.0 pixels (large node)
- When: `touch_hit_radius(50.0, true)` is called (touch input)
- Then: Returns max(50.0, 44.0) = 50.0 (larger base wins)

**test_sel_008_touch_handle_hit_area_extends_for_accessibility**
- Given: Handle at position (100, 100), base radius 10px
- When: `touch_handle_hit_test(105, 105, 100, 100, true)` is called
- Then: Returns true (within extended touch hit area)

**test_sel_008_mouse_hit_radius_equals_base**
- Given: Base hit radius of 10.0 pixels
- When: `touch_hit_radius(10.0, false)` is called (mouse input)
- Then: Returns 10.0 (no extension)

**test_sel_008_touch_hit_area_larger_than_mouse**
- Given: Touch point at (141, 141), node center at (100, 100) (41px distance)
- When: Testing hit with touch vs mouse (base_radius = 30)
- Then: Touch returns true (41 < 44), mouse returns false (41 > 30)

### SEL-009: Drag Threshold Prevents Accidental Drag

**test_sel_009_movement_below_threshold_returns_false**
- Given: Origin at (0, 0)
- When: `has_drag_threshold((0.0, 0.0), (2.9, 0.0))` is called
- Then: Returns false

**test_sel_009_movement_at_threshold_returns_true**
- Given: Origin at (0, 0)
- When: `has_drag_threshold((0.0, 0.0), (3.0, 0.0))` is called
- Then: Returns true

**test_sel_009_movement_above_threshold_returns_true**
- Given: Origin at (0, 0)
- When: `has_drag_threshold((0.0, 0.0), (10.0, 0.0))` is called
- Then: Returns true

**test_sel_009_diagonal_movement_uses_euclidean_distance**
- Given: Origin at (0, 0)
- When: `has_drag_threshold((0.0, 0.0), (2.0, 2.0))` is called (distance ≈ 2.83)
- Then: Returns false (under 3.0 threshold)

**test_sel_009_diagonal_movement_above_threshold**
- Given: Origin at (0, 0)
- When: `has_drag_threshold((0.0, 0.0), (2.0, 3.0))` is called (distance ≈ 3.61)
- Then: Returns true

**test_sel_009_threshold_symmetric_regardless_of_direction**
- Given: Origin at (100, 100)
- When: Testing threshold in all 4 directions and diagonals
- Then: All return same result for same distance

---

## Error Path Tests

### SEL-006: Hover
**test_sel_006_returns_error_for_nonexistent_node**
- Given: A document without Node X
- When: Hover is attempted on Node X
- Then: Returns `Err(SelectionError::NodeNotFound)`

### SEL-009: Drag Threshold
**test_sel_009_threshold_check_handles_nan_gracefully**
- Given: Origin at (0, 0), current position with NaN
- When: `has_drag_threshold` is called
- Then: Returns false (or handles gracefully without panic)

---

## Edge Case Tests

### SEL-006: Hover
**test_sel_006_hover_handles_overlapping_nodes**
- Given: Two overlapping nodes, A on top of B (higher z-index)
- When: Hover at the overlap position
- Then: Top node (A) is hovered, not B

**test_sel_006_hover_transitions_from_idle**
- Given: Initial state is Idle
- When: Mouse enters a node
- Then: Transitions through valid state machine to Hovering

### SEL-007: Handles
**test_sel_007_handles_at_extreme_coordinates**
- Given: Node at very large coordinates (10000, 10000)
- When: Handles are computed
- Then: All 8 handles are at valid positions within bounds

**test_sel_007_handles_for_zero_size_selection**
- Given: Selection bounds with zero width or height
- When: Handles are computed
- Then: Returns 8 handles (handled by invariant I1)

### SEL-008: Touch
**test_sel_008_touch_hit_radius_never_negative**
- Given: Any base radius value (including 0 or negative edge cases)
- When: `touch_hit_radius` is called with is_touch = true
- Then: Returns value >= 44.0 (WCAG minimum)

**test_sel_008_touch_extension_at_boundary**
- Given: Base radius exactly equal to 44.0
- When: Touch hit radius computed
- Then: Returns 44.0 (boundary case)

**test_sel_008_touch_extension_above_boundary**
- Given: Base radius > 44.0 (e.g., 50.0)
- When: Touch hit radius computed
- Then: Returns base value (50.0) - larger base wins

### SEL-009: Drag Threshold
**test_sel_009_zero_origin_handled**
- Given: Origin at (0, 0)
- When: `has_drag_threshold((0.0, 0.0), (0.0, 0.0))` is called
- Then: Returns false

**test_sel_009_negative_coordinates_handled**
- Given: Origin at (-100, -100)
- When: Threshold check with various positions
- Then: Returns correct result using Euclidean distance

---

## Contract Verification Tests

### SEL-005: Marquee Direction (Reference - from seshat-9n1)
**test_contract_p1_marquee_left_to_right_uses_containment**
- Given: Node at (10, 10) size 50x50
- When: Marquee from (0, 0) to (30, 30) (L→R, node partially inside)
- Then: Node NOT selected (not fully contained)

**test_contract_p2_marquee_right_to_left_uses_intersection**
- Given: Node at (10, 10) size 50x50
- When: Marquee from (30, 30) to (0, 0) (R→L)
- Then: Node IS selected (intersects)

### SEL-006: Hover
**test_contract_q2_hovered_node_id_stored**
- Given: A node exists
- When: Hover begins on node
- Then: `hovered_node.read()` returns Some(node_id)

**test_contract_q3_hover_state_transition**
- Given: Hover state is None
- When: Mouse enters node bounds
- Then: Hover state becomes Some(node_id)

### SEL-007: Resize Handles
**test_contract_q5_eight_handles_for_non_empty_selection**
- Given: At least one node selected
- When: Handles computed
- Then: Exactly 8 handles returned

### SEL-008: Touch (FIXED - WCAG 44px)
**test_contract_q8_touch_radius_for_nodes_uses_44px_minimum**
- Given: is_touch = true
- When: `touch_hit_radius(10.0, true)` called
- Then: Returns max(10.0, 44.0) = 44.0 (WCAG minimum)

**test_contract_q9_touch_radius_for_handles_uses_44px_minimum**
- Given: is_touch = true, handle base size 14.0
- When: Handle touch hit area computed
- Then: Returns max(14.0, 44.0) = 44.0 (exceeds handle size)

**test_contract_q10_mouse_radius_unchanged**
- Given: is_touch = false
- When: `touch_hit_radius(10.0, false)` called
- Then: Returns 10.0

### SEL-009: Drag Threshold
**test_contract_q11_threshold_under_3px**
- Given: Movement distance < 3.0
- When: `has_drag_threshold` called
- Then: Returns false

**test_contract_q12_threshold_at_3px**
- Given: Movement distance = 3.0
- When: `has_drag_threshold` called
- Then: Returns true

**test_contract_q13_euclidean_for_diagonal**
- Given: Movement (2.0, 2.0)
- When: Threshold check
- Then: Uses sqrt(2² + 2²) = 2.83, returns false

---

## Contract Violation Tests

### Violation P3 (SEL-006)
**test_p3_violation_returns_node_not_found_error**
- Given: Document without NodeId("nonexistent")
- When: Hover is attempted on NodeId("nonexistent")
- Then: Returns `Err(SelectionError::NodeNotFound)` -- NOT a panic

### Violation Q8 (SEL-008) - FIXED
**test_q8_violation_touch_returns_44px_for_touch_input**
- Given: is_touch = true, base_radius = 10.0
- When: `touch_hit_radius(10.0, true)` called
- Then: Returns 44.0 (not 10.0 or 8.0)

### Violation Q9 (SEL-008) - FIXED
**test_q9_violation_handle_touch_uses_44px**
- Given: is_touch = true, handle size 14.0
- When: Handle touch hit area computed
- Then: Returns 44.0 (not 14.0 or 22.0)

### Violation Q11 (SEL-009)
**test_q11_violation_under_threshold_returns_false**
- Given: Origin (0, 0), current (2.9, 0)
- When: `has_drag_threshold` called
- Then: Returns false -- NOT true

### Violation Q12 (SEL-009)
**test_q12_violation_at_threshold_returns_true**
- Given: Origin (0, 0), current (3.0, 0)
- When: `has_drag_threshold` called
- Then: Returns true -- NOT false

### Violation Q13 (SEL-009)
**test_q13_violation_diagonal_uses_euclidean**
- Given: Origin (0, 0), current (2.0, 2.0)
- When: `has_drag_threshold` called
- Then: Returns false (sqrt(8) < 3.0) -- NOT true (which would use Manhattan)

---

## Given-When-Then Scenarios

### Scenario 1: SEL-006 Hover Feedback
**Scenario: User hovers over a node and sees visual feedback**
- Given: A document with Node A (x: 0, y: 0, w: 100, h: 100)
- When: Mouse enters Node A's bounds
- Then:
  - Hover state becomes Hovering(NodeId("n1"))
  - Visual border style changes (tested via UI, not unit test)
  - Hover state persists while mouse moves within bounds
- When: Mouse leaves Node A's bounds
- Then: Hover state returns to Idle

### Scenario 2: SEL-007 Resize Handle Interaction
**Scenario: User drags a resize handle to resize selection**
- Given: Node A is selected (bounds: 0, 0, 100, 100)
- When: User clicks on the SE handle
- Then:
  - Handle is identified as SE
  - Drag operation begins from handle position
  - Resize updates selection bounds proportionally

### Scenario 3: SEL-008 Touch Input Selection (WCAG)
**Scenario: User taps on a node using touch screen**
- Given: Node A exists at (100, 100) with size 50x50
- And: Input device is touch (is_touch = true)
- When: User taps at (120, 120) - 20px outside visual bounds
- Then: Node A is selected (touch hit area extends to 44px radius = 22px from center)
- Note: At center (100,100), a 44px radius covers (78,78) to (122,122), so (120,120) is inside

### Scenario 4: SEL-009 Drag Threshold
**Scenario: User clicks on node but doesn't drag enough to move**
- Given: Node A is selected at (100, 100)
- When: User clicks and moves mouse only 2px
- Then:
  - `has_drag_threshold` returns false
  - No move operation occurs
  - Node remains at original position
- When: User clicks and moves mouse 5px
- Then:
  - `has_drag_threshold` returns true
  - Move operation begins

---

## Testing Trophy Integration

This test plan addresses the Testing Trophy through layered test coverage:

### Unit Tests (This Bead - seshat-zzn)
- Core algorithmic functions: `touch_hit_radius`, `has_drag_threshold`, handle hit testing
- Mathematical correctness: Euclidean distance, boundary conditions
- Pure functions with deterministic outputs

### Integration Tests (Separate Bead)
- Component interactions: SelectionState ↔ HoverState ↔ DragState
- Document mutation: Selection changes affecting document model
- Event handling: Pointer events triggering state transitions

### E2E Tests (Separate Bead)
- Browser-based touch selection
- Visual feedback rendering
- Complete user workflows

---

## Test Implementation Notes

1. **Fixtures**: Use the existing `setup_doc()` pattern from selection_ops_tests.rs
2. **Naming**: Follow pattern `test_sel_XXX_description`
3. **Assertions**: Use `assert!` and `assert_eq!` with descriptive messages
4. **Error Cases**: Verify `Err` variant matches, not just that error occurred
5. **Constants**: Use `TOUCH_HIT_RADIUS_PX = 44.0` and `TOUCH_HIT_RADIUS_MIN = 22.0`
6. **Integration**: These are unit tests; UI rendering tests are out of scope

(End of file - total 365 lines)
