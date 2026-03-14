# Martin Fowler Test Plan

## Happy Path Tests

### test_line_line_intersects_returns_true_for_crossing_segments
Given: Two line segments that cross at their midpoints
When: Calling line_line_intersects
Then: Returns true

### test_line_line_intersection_returns_point_for_crossing_segments
Given: Two line segments that cross at their midpoints
When: Calling line_line_intersection
Then: Returns Some(Point) at the intersection coordinates

### test_line_line_intersects_returns_false_for_parallel_lines
Given: Two parallel horizontal lines
When: Calling line_line_intersects
Then: Returns false

### test_line_rect_intersects_returns_true_for_line_crossing_rect
Given: A line segment that crosses through a rectangle
When: Calling line_rect_intersects
Then: Returns true

### test_line_rect_intersections_returns_two_points_for_crossing_line
Given: A line segment that crosses through a rectangle (entering and exiting)
When: Calling line_rect_intersections
Then: Returns exactly 2 points

## Error Path Tests

### test_line_segment_new_rejects_nan_coordinates
Given: A point with NaN x-coordinate
When: Creating LineSegment::new
Then: Returns Err(Error::InvalidEndpoint)

### test_line_segment_new_rejects_infinity_coordinates
Given: A point with Infinity y-coordinate
When: Creating LineSegment::new
Then: Returns Err(Error::InvalidEndpoint)

### test_line_segment_new_rejects_zero_length_segment
Given: Start and end points are identical
When: Creating LineSegment::new
Then: Returns Err(Error::DegenerateLine)

### test_line_line_intersection_returns_none_for_parallel_lines
Given: Two parallel lines that never meet
When: Calling line_line_intersection
Then: Returns None

### test_line_line_intersection_returns_none_for_disjoint_segments
Given: Two line segments that don't touch
When: Calling line_line_intersection
Then: Returns None

## Edge Case Tests

### test_line_line_intersects_handles_endpoint_touching
Given: Two line segments that touch at an endpoint
When: Calling line_line_intersects
Then: Returns true (intersection at common endpoint)

### test_line_line_intersection_handles_collinear_overlapping
Given: Two collinear line segments that overlap
When: Calling line_line_intersection
Then: Returns the overlapping segment (implementation-defined: first point of overlap)

### test_line_rect_intersects_returns_false_for_line_outside_rect
Given: A line segment completely outside the rectangle
When: Calling line_rect_intersects
Then: Returns false

### test_line_rect_intersects_returns_true_for_line_touching_corner
Given: A line segment that touches a rectangle corner
When: Calling line_rect_intersects
Then: Returns true

### test_line_rect_intersections_returns_one_point_for_tangent
Given: A line segment tangent to the rectangle (touches one edge)
When: Calling line_rect_intersections
Then: Returns 1 point

### test_line_rect_intersects_handles_vertical_line
Given: A vertical line crossing a rectangle
When: Calling line_rect_intersects
Then: Returns true

### test_line_rect_intersects_handles_horizontal_line
Given: A horizontal line crossing a rectangle
When: Calling line_rect_intersects
Then: Returns true

## Contract Verification Tests

### test_precondition_p1_rejects_nan_coordinates
Given: Line segment with NaN coordinates
When: Creating LineSegment
Then: Returns error (precondition P1 enforced)

### test_precondition_p2_rejects_zero_length
Given: Identical start and end points
When: Creating LineSegment
Then: Returns error (precondition P2 enforced)

### test_postcondition_q1_intersects_consistency
Given: Any two line segments a, b
When: Calling both line_line_intersects and line_line_intersection
Then: line_line_intersection returns Some iff line_line_intersects returns true (Q1/Q2 consistency)

### test_invariant_i1_intersection_points_on_segment
Given: Valid intersection from line_line_intersection
When: Checking if point lies on both segments
Then: Returns point within epsilon tolerance of both segments (invariant I1)

## Given-When-Then Scenarios

### Scenario 1: Connector routing avoids obstacle
Given: A source point at (0, 50), target point at (100, 50), and rectangular obstacle at (40, 30) to (60, 70)
When: Checking if direct path intersects obstacle
Then: Returns true, and route must be recalculated

### Scenario 2: Line passes through multiple rectangles
Given: A line from (0, 0) to (100, 100), intersecting rect1 at (20, 20)-(40, 40) and rect2 at (60, 60)-(80, 80)
When: Finding all intersection points
Then: Returns 4 points (2 per rectangle)

### Scenario 3: Diagonal connector with corner snap
Given: A diagonal line from (10, 10) to (90, 90), rectangle at (30, 30)-(70, 70)
When: Computing intersections
Then: Returns exactly 2 points on the rectangle's top-right and bottom-left edges
