# Martin Fowler Test Plan

## Overview
This test plan covers GEO-020: Hit test margin respects zoom level.

The specified behavior is **screen-space**: the hit margin remains constant in screen pixels regardless of zoom level. This ensures users can reliably click near node edges whether zoomed in or out.

## Test IDs
- GEO-020-T001 through GEO-020-T034

## Behavior-Driven Tests (User-Facing Gherkin)

### GEO-020-T001: Easy node selection when zoomed out
**User Story**: As a user, I want to easily select nodes when zoomed out
**Given**: A diagram with a node at screen position (0,0) to (100,100)
**And**: Zoom level is 0.1 (zoomed far out)
**When**: I click 5 pixels from the node edge
**Then**: The node is selected (system accounts for zoom with larger hit area)

### GEO-020-T002: Precise node selection when zoomed in
**User Story**: As a user, I want precise selection when zoomed in
**Given**: A diagram with a node at screen position (0,0) to (100,100)
**And**: Zoom level is 4.0 (zoomed in close)
**When**: I click 5 pixels from the node edge
**Then**: The node is NOT selected (tighter hit area at high zoom)

### GEO-020-T003: Consistent selection at default zoom
**User Story**: As a user, I expect normal selection at default zoom
**Given**: A diagram with a node at screen position (0,0) to (100,100)
**And**: Zoom level is 1.0 (default)
**When**: I click 5 pixels from the node edge
**Then**: The node is selected

### GEO-020-T004: Hit test margin scales with zoom inversely
**User Story**: As a user, I want the hit area to stay the same size on screen
**Given**: A 5-pixel screen margin configured
**When**: Computing world-space margin at different zoom levels
**Then**: At zoom 0.1: 50.0 world units, at zoom 1.0: 5.0 world units, at zoom 4.0: 1.25 world units

## Error Path Tests

### GEO-020-T010: Reject zoom below minimum
**Given**: zoom = 0.05 (below MIN_ZOOM of 0.1)
**When**: computing world margin
**Then**: returns error indicating invalid zoom

### GEO-020-T011: Reject zoom above maximum
**Given**: zoom = 5.0 (above MAX_ZOOM of 4.0)
**When**: computing world margin
**Then**: returns error indicating invalid zoom

### GEO-020-T012: Reject negative zoom
**Given**: zoom = -1.0
**When**: computing world margin
**Then**: returns error indicating invalid zoom

### GEO-020-T013: Reject zero margin
**Given**: screen_margin = 0.0
**When**: computing world margin
**Then**: returns error indicating invalid margin

### GEO-020-T014: Reject negative margin
**Given**: screen_margin = -5.0
**When**: computing world margin
**Then**: returns error indicating invalid margin

### GEO-020-T015: Reject NaN point coordinates
**Given**: point with NaN coordinates
**When**: performing hit test
**Then**: returns error indicating invalid point

### GEO-020-T016: Reject infinite point coordinates
**Given**: point with infinite coordinates
**When**: performing hit test
**Then**: returns error indicating invalid point

## Integration Tests (Actual Test References)

These tests reference the actual integration tests that exercise this feature:

### GEO-020-T040: Integration test - zoomed out viewport selection
**Reference**: `diagram_tool/src/geometry/hit_test_tests.rs` - tests zoom=0.1 hit detection
**Verifies**: Users can select nodes easily when zoomed out

### GEO-020-T041: Integration test - zoomed in viewport selection  
**Reference**: `diagram_tool/src/geometry/hit_test_tests.rs` - tests zoom=4.0 hit detection
**Verifies**: Users get precise selection when zoomed in

### GEO-020-T042: Integration test - IO serialization
**Reference**: `diagram_tool/src/models/io_tests.rs` - tests for serialization of hit test parameters
**Verifies**: Hit margin configuration persists correctly

## Edge Case Tests

### GEO-020-T020: Zoom at exact minimum boundary
**Given**: zoom = MIN_ZOOM (0.1)
**When**: computing world margin
**Then**: returns correctly computed value without panicking

### GEO-020-T021: Zoom at exact maximum boundary
**Given**: zoom = MAX_ZOOM (4.0)
**When**: computing world margin
**Then**: returns correctly computed value without panicking

### GEO-020-T022: Very small screen margin
**Given**: screen_margin = 0.001
**When**: computing world margin
**Then**: returns approximately 0.001 / zoom

### GEO-020-T023: Very large screen margin
**Given**: screen_margin = 10000.0
**When**: computing world margin at min zoom
**Then**: returns 100000.0 without overflow

### GEO-020-T024: Point exactly on margin boundary
**Given**: point exactly at margin distance from rect edge
**When**: performing hit test
**Then**: returns true (boundary is inclusive)

### GEO-020-T025: Point just outside margin
**Given**: point just outside margin distance from rect edge
**When**: performing hit test
**Then**: returns false

## Contract Verification Tests

### GEO-020-T030: Verify postcondition Q1 at min zoom
**Given**: zoom = 0.1
**When**: calling screen_to_world_margin(5.0, zoom)
**Then**: result must equal 50.0 (per Q1)

### GEO-020-T031: Verify postcondition Q2 at max zoom
**Given**: zoom = 4.0
**When**: calling screen_to_world_margin(5.0, zoom)
**Then**: result must equal 1.25 (per Q2)

### GEO-020-T032: Verify postcondition Q3 at unit zoom
**Given**: zoom = 1.0
**When**: calling screen_to_world_margin(5.0, zoom)
**Then**: result must equal 5.0 (per Q3)

### GEO-020-T033: Verify invariant I1 - screen-space consistency
**Given**: A point at fixed screen distance from edge
**When**: performing hit test at different zoom levels (0.1, 1.0, 4.0)
**Then**: All return same hit result (consistent screen-space behavior)

### GEO-020-T034: Verify invariant I2 - world margin decreases with zoom
**Given**: A 5-pixel screen margin
**When**: Computing world margin at zoom 0.1, 1.0, and 4.0
**Then**: Values are 50.0, 5.0, and 1.25 respectively (world margin decreases as zoom increases)

## Contract Violation Tests

### GEO-020-V001: Precondition P1 violation - zoom below minimum
**Given**: zoom = 0.05 (below MIN_ZOOM)
**When**: screen_to_world_margin(5.0, 0.05)
**Then**: returns error -- NOT a panic

### GEO-020-V002: Precondition P1 violation - zoom above maximum
**Given**: zoom = 5.0 (above MAX_ZOOM)
**When**: screen_to_world_margin(5.0, 5.0)
**Then**: returns error -- NOT a panic

### GEO-020-V003: Precondition P2 violation - zero margin
**Given**: margin = 0.0
**When**: screen_to_world_margin(0.0, 1.0)
**Then**: returns error -- NOT a panic

### GEO-020-V004: Precondition P2 violation - negative margin
**Given**: margin = -5.0
**When**: screen_to_world_margin(-5.0, 1.0)
**Then**: returns error -- NOT a panic

### GEO-020-V005: Precondition P3 violation - NaN point
**Given**: point with NaN x-coordinate
**When**: hit test with invalid point
**Then**: returns error -- NOT a panic

## User Scenarios (Given-When-Then)

### Scenario 1: Selecting a node while zoomed out
**Given**: A diagram with a node, user has zoomed out to 0.1x
**When**: User clicks 5 screen pixels from the node edge
**Then**: The node is selected because the hit area is large (50 world units)

### Scenario 2: Precise selection while zoomed in  
**Given**: A diagram with a node, user has zoomed in to 4.0x
**When**: User clicks 5 screen pixels from the node edge
**Then**: The node is NOT selected because the hit area is small (1.25 world units)

### Scenario 3: Missing the node entirely
**Given**: A rectangle at (0,0) with size 100x100, zoom = 1.0
**When**: User clicks at (10.0, 50.0) which is 10 units from the edge
**Then**: The hit test fails because margin is only 5.0

### Scenario 4: Hitting the exact boundary
**Given**: A rectangle at (0,0) with size 100x100, zoom = 1.0, 5-pixel margin
**When**: User clicks exactly 5 units from the edge
**Then**: The hit test succeeds (boundary is inclusive)

## Test Execution Order
1. Error path tests (T010-T016) - verify invalid inputs are handled
2. Happy path tests (T001-T004) - verify basic functionality
3. Edge case tests (T020-T025) - verify boundary conditions
4. Contract verification tests (T030-T034) - verify postconditions and invariants
5. Contract violation tests (V001-V005) - verify error handling

## Integration Test References
- `diagram_tool/src/geometry/hit_test_tests.rs` - Core hit test logic with zoom
- `diagram_tool/src/models/io_tests.rs` - IO-001 to IO-015 coverage for serialization
- Run with: `moon run diagram_tool:test`
