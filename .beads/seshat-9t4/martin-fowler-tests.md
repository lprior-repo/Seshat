# Martin Fowler Test Plan

## Overview
This test plan covers GEO-020: Hit test margin should either be constant screen-space or constant world-space depending on spec.

## Test IDs
- GEO-020-T001 through GEO-020-T025

## Happy Path Tests

### GEO-020-T001: screen_to_world_margin_at_min_zoom
**Given**: screen_margin = 5.0, zoom = MIN_ZOOM (0.1)
**When**: computing world margin from screen margin
**Then**: returns 50.0 (5.0 / 0.1)
```
assert_eq!(screen_to_world_margin(5.0, 0.1), Ok(50.0));
```

### GEO-020-T002: screen_to_world_margin_at_max_zoom
**Given**: screen_margin = 5.0, zoom = MAX_ZOOM (4.0)
**When**: computing world margin from screen margin
**Then**: returns 1.25 (5.0 / 4.0)
```
assert_eq!(screen_to_world_margin(5.0, 4.0), Ok(1.25));
```

### GEO-020-T003: screen_to_world_margin_at_unit_zoom
**Given**: screen_margin = 5.0, zoom = 1.0
**When**: computing world margin from screen margin
**Then**: returns 5.0 (5.0 / 1.0)
```
assert_eq!(screen_to_world_margin(5.0, 1.0), Ok(5.0));
```

### GEO-020-T004: screen_to_world_margin_intermediate_zoom
**Given**: screen_margin = 5.0, zoom = 2.0
**When**: computing world margin from screen margin
**Then**: returns 2.5 (5.0 / 2.0)
```
assert_eq!(screen_to_world_margin(5.0, 2.0), Ok(2.5));
```

### GEO-020-T005: hit_test_with_margin_at_min_zoom
**Given**: point at (5.0, 50.0), rectangle at (0,0) with size 100x100, zoom = 0.1
**When**: performing hit test with screen-space margin
**Then**: returns true (point within expanded rect)
```
let rect = Rectangle::new(0.0, 0.0, 100.0, 100.0);
let point = Point::new(5.0, 50.0); // 5 world units from edge
let hit = hit_test_with_zoom_margin(point, &rect, 0.1, MarginBehavior::ScreenSpace)?;
assert!(hit);
```

### GEO-020-T006: hit_test_with_margin_at_max_zoom
**Given**: point at (5.0, 50.0), rectangle at (0,0) with size 100x100, zoom = 4.0
**When**: performing hit test with screen-space margin
**Then**: returns true (point within tighter expanded rect at high zoom)
```
let rect = Rectangle::new(0.0, 0.0, 100.0, 100.0);
let point = Point::new(5.0, 50.0);
let hit = hit_test_with_zoom_margin(point, &rect, 4.0, MarginBehavior::ScreenSpace)?;
assert!(hit);
```

### GEO-020-T007: world_margin_constant_at_any_zoom
**Given**: world_margin = 10.0, zoom varies (0.1, 1.0, 4.0)
**When**: using world-space constant behavior
**Then**: returns 10.0 for all zoom values
```
assert_eq!(world_margin_constant(10.0, 0.1), Ok(10.0));
assert_eq!(world_margin_constant(10.0, 1.0), Ok(10.0));
assert_eq!(world_margin_constant(10.0, 4.0), Ok(10.0));
```

### GEO-020-T008: hit_test_point_inside_rectangle_no_margin
**Given**: point at (50.0, 50.0), rectangle at (0,0) with size 100x100
**When**: performing hit test with zero margin
**Then**: returns true
```
let rect = Rectangle::new(0.0, 0.0, 100.0, 100.0);
let point = Point::new(50.0, 50.0);
let hit = hit_test_with_zoom_margin(point, &rect, 1.0, MarginBehavior::ScreenSpace)?;
assert!(hit);
```

## Error Path Tests

### GEO-020-T010: invalid_zoom_below_min
**Given**: zoom = 0.05 (below MIN_ZOOM)
**When**: computing world margin
**Then**: returns Err(HitTestError::InvalidZoom)
```
assert_eq!(screen_to_world_margin(5.0, 0.05), Err(HitTestError::InvalidZoom));
```

### GEO-020-T011: invalid_zoom_above_max
**Given**: zoom = 5.0 (above MAX_ZOOM)
**When**: computing world margin
**Then**: returns Err(HitTestError::InvalidZoom)
```
assert_eq!(screen_to_world_margin(5.0, 5.0), Err(HitTestError::InvalidZoom));
```

### GEO-020-T012: invalid_zoom_negative
**Given**: zoom = -1.0
**When**: computing world margin
**Then**: returns Err(HitTestError::InvalidZoom)
```
assert_eq!(screen_to_world_margin(5.0, -1.0), Err(HitTestError::InvalidZoom));
```

### GEO-020-T013: invalid_margin_zero
**Given**: screen_margin = 0.0
**When**: computing world margin
**Then**: returns Err(HitTestError::InvalidMargin)
```
assert_eq!(screen_to_world_margin(0.0, 1.0), Err(HitTestError::InvalidMargin));
```

### GEO-020-T014: invalid_margin_negative
**Given**: screen_margin = -5.0
**When**: computing world margin
**Then**: returns Err(HitTestError::InvalidMargin)
```
assert_eq!(screen_to_world_margin(-5.0, 1.0), Err(HitTestError::InvalidMargin));
```

### GEO-020-T015: invalid_point_nan
**Given**: point with NaN coordinates
**When**: performing hit test
**Then**: returns Err(HitTestError::InvalidPoint)
```
let point = Point::new(f64::NAN, 50.0);
assert_eq!(hit_test_with_zoom_margin(point, &rect, 1.0, MarginBehavior::ScreenSpace), Err(HitTestError::InvalidPoint));
```

### GEO-020-T016: invalid_point_infinity
**Given**: point with infinite coordinates
**When**: performing hit test
**Then**: returns Err(HitTestError::InvalidPoint)
```
let point = Point::new(f64::INFINITY, 50.0);
assert_eq!(hit_test_with_zoom_margin(point, &rect, 1.0, MarginBehavior::ScreenSpace), Err(HitTestError::InvalidPoint));
```

## Edge Case Tests

### GEO-020-T020: zoom_at_exact_min_boundary
**Given**: zoom = MIN_ZOOM (0.1)
**When**: computing world margin
**Then**: returns correctly computed value without panicking
```
let result = screen_to_world_margin(5.0, MIN_ZOOM);
assert!(result.is_ok());
```

### GEO-020-T021: zoom_at_exact_max_boundary
**Given**: zoom = MAX_ZOOM (4.0)
**When**: computing world margin
**Then**: returns correctly computed value without panicking
```
let result = screen_to_world_margin(5.0, MAX_ZOOM);
assert!(result.is_ok());
```

### GEO-020-T022: very_small_screen_margin
**Given**: screen_margin = 0.001
**When**: computing world margin
**Then**: returns approximately 0.001 / zoom
```
let result = screen_to_world_margin(0.001, 1.0)?;
assert!((result - 0.001).abs() < 1e-10);
```

### GEO-020-T023: very_large_screen_margin
**Given**: screen_margin = 10000.0
**When**: computing world margin at min zoom
**Then**: returns 100000.0 without overflow
```
let result = screen_to_world_margin(10000.0, 0.1)?;
assert!((result - 100000.0).abs() < f64::EPSILON);
```

### GEO-020-T024: point_exactly_on_margin_boundary
**Given**: point exactly at margin distance from rect edge
**When**: performing hit test
**Then**: returns true (boundary is inclusive)
```
let rect = Rectangle::new(0.0, 0.0, 100.0, 100.0);
let point = Point::new(5.0, 50.0); // exactly at margin=5
let hit = hit_test_with_zoom_margin(point, &rect, 1.0, MarginBehavior::ScreenSpace)?;
assert!(hit);
```

### GEO-020-T025: point_just_outside_margin
**Given**: point just outside margin distance from rect edge
**When**: performing hit test
**Then**: returns false
```
let rect = Rectangle::new(0.0, 0.0, 100.0, 100.0);
let point = Point::new(5.1, 50.0); // just outside margin=5
let hit = hit_test_with_zoom_margin(point, &rect, 1.0, MarginBehavior::ScreenSpace)?;
assert!(!hit);
```

## Contract Verification Tests

### GEO-020-T030: verify_postcondition_q1_min_zoom
**Given**: zoom = 0.1
**When**: calling screen_to_world_margin(5.0, zoom)
**Then**: result must equal 50.0 (per Q1)
```
let result = screen_to_world_margin(5.0, 0.1)?;
assert!((result - 50.0).abs() < TOLERANCE);
```

### GEO-020-T031: verify_postcondition_q2_max_zoom
**Given**: zoom = 4.0
**When**: calling screen_to_world_margin(5.0, zoom)
**Then**: result must equal 1.25 (per Q2)
```
let result = screen_to_world_margin(5.0, 4.0)?;
assert!((result - 1.25).abs() < TOLERANCE);
```

### GEO-020-T032: verify_postcondition_q3_unit_zoom
**Given**: zoom = 1.0
**When**: calling screen_to_world_margin(5.0, zoom)
**Then**: result must equal 5.0 (per Q3)
```
let result = screen_to_world_margin(5.0, 1.0)?;
assert!((result - 5.0).abs() < TOLERANCE);
```

### GEO-020-T033: verify_invariant_i1_screen_space_consistency
**Given**: A point at fixed screen distance from edge, varying zoom
**When**: performing hit test at zoom 0.1, 1.0, 4.0
**Then**: all return same hit result (true or false consistently)
```
// At screen distance 10px from edge: world distances are 100, 10, 2.5 at respective zooms
let rect = Rectangle::new(0.0, 0.0, 100.0, 100.0);
// 10 screen pixels = 100 world at zoom 0.1
let hit_low = hit_test_with_zoom_margin(Point::new(10.0, 50.0), &rect, 0.1, MarginBehavior::ScreenSpace)?;
// 10 screen pixels = 10 world at zoom 1.0  
let hit_mid = hit_test_with_zoom_margin(Point::new(10.0, 50.0), &rect, 1.0, MarginBehavior::ScreenSpace)?;
// 10 screen pixels = 2.5 world at zoom 4.0
let hit_high = hit_test_with_zoom_margin(Point::new(2.5, 50.0), &rect, 4.0, MarginBehavior::ScreenSpace)?;
assert_eq!(hit_low, hit_mid); // both should hit
```

## Contract Violation Tests

### GEO-020-V001: precondition_p1_violation_below_min_zoom
**Given**: zoom = 0.05 (below MIN_ZOOM)
**When**: screen_to_world_margin(5.0, 0.05)
**Then**: returns Err(HitTestError::InvalidZoom) -- NOT a panic
```
// VIOLATES P1
assert_eq!(screen_to_world_margin(5.0, 0.05), Err(HitTestError::InvalidZoom));
```

### GEO-020-V002: precondition_p1_violation_above_max_zoom
**Given**: zoom = 5.0 (above MAX_ZOOM)
**When**: screen_to_world_margin(5.0, 5.0)
**Then**: returns Err(HitTestError::InvalidZoom) -- NOT a panic
```
// VIOLATES P1
assert_eq!(screen_to_world_margin(5.0, 5.0), Err(HitTestError::InvalidZoom));
```

### GEO-020-V003: precondition_p2_violation_zero_margin
**Given**: margin = 0.0
**When**: screen_to_world_margin(0.0, 1.0)
**Then**: returns Err(HitTestError::InvalidMargin) -- NOT a panic
```
// VIOLATES P2
assert_eq!(screen_to_world_margin(0.0, 1.0), Err(HitTestError::InvalidMargin));
```

### GEO-020-V004: precondition_p2_violation_negative_margin
**Given**: margin = -5.0
**When**: screen_to_world_margin(-5.0, 1.0)
**Then**: returns Err(HitTestError::InvalidMargin) -- NOT a panic
```
// VIOLATES P2
assert_eq!(screen_to_world_margin(-5.0, 1.0), Err(HitTestError::InvalidMargin));
```

### GEO-020-V005: precondition_p3_violation_nan_point
**Given**: point with NaN x-coordinate
**When**: hit_test_with_zoom_margin(Point::new(f64::NAN, 50.0), &rect, 1.0, MarginBehavior::ScreenSpace)
**Then**: returns Err(HitTestError::InvalidPoint) -- NOT a panic
```
// VIOLATES P3
assert_eq!(hit_test_with_zoom_margin(Point::new(f64::NAN, 50.0), &rect, 1.0, MarginBehavior::ScreenSpace), Err(HitTestError::InvalidPoint));
```

## Given-When-Then Scenarios

### Scenario 1: Screen-space hit margin at minimum zoom
**Given**: A diagram with a node at (0,0) with size 100x100, zoom = 0.1 (far away)
**When**: User clicks 5 screen pixels from the node edge
**Then**: The hit test succeeds because the world margin is 50.0 (large hit area when zoomed out)

### Scenario 2: Screen-space hit margin at maximum zoom
**Given**: A diagram with a node at (0,0) with size 100x100, zoom = 4.0 (close up)
**When**: User clicks the same 5 screen pixels from the node edge
**Then**: The hit test succeeds because the world margin is 1.25 (smaller hit area when zoomed in)

### Scenario 3: World-space hit margin consistency
**Given**: A diagram with a node, zoom = 0.1 and zoom = 4.0
**When**: User clicks at the same world distance from the node edge
**Then**: Both hit tests return the same result regardless of zoom level

### Scenario 4: Hit test miss outside margin
**Given**: A rectangle at (0,0) with size 100x100, zoom = 1.0
**When**: User clicks at (10.0, 50.0) which is 10 units from the edge
**Then**: The hit test fails because margin is only 5.0

## Test Execution Order
1. Error path tests (T010-T016) - verify invalid inputs are handled
2. Happy path tests (T001-T008) - verify basic functionality
3. Edge case tests (T020-T025) - verify boundary conditions
4. Contract verification tests (T030-T033) - verify postconditions
5. Contract violation tests (V001-V005) - verify error handling
