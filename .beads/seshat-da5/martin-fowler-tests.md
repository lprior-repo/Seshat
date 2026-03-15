# Martin Fowler Test Plan: AABB includes stroke width (GEO-003)

## Happy Path Tests

### test_aabb_basic_construction
Given: Valid bounds (min_x=0, min_y=0, max_x=100, max_y=50)
When: Creating AABB with AABB::new
Then: Returns valid AABB with correct min/max values

### test_aabb_expand_by_positive_amount
Given: AABB(0, 0, 100, 100)
When: Expanding by 10 units
Then: Returns AABB(-10, -10, 110, 110)

### test_aabb_expand_preserves_center
Given: AABB(0, 0, 100, 100)
When: Expanding by 10 units
Then: Center remains at (50, 50)

### test_bounds_with_stroke_rectangle
Given: Rectangle at (0, 0) with width=100, height=50 and stroke_width=4
When: Calling bounds_with_stroke()
Then: Returns AABB expanded by stroke_width/2 = 2 on each side: (-2, -2, 102, 52)

### test_bounds_with_hit_margin
Given: AABB(0, 0, 100, 100)
When: Adding hit margin of 5 units
Then: Returns AABB(-5, -5, 105, 105)

### test_combined_stroke_and_hit_margin
Given: Rectangle at (0, 0) with width=100, height=50, stroke_width=4, hit_margin=5
When: Getting bounds with both stroke and hit margin
Then: Returns AABB expanded by stroke/2 + margin = 2 + 5 = 7: (-7, -7, 107, 57)

### test_rectangle_at_various_positions
Given: Rectangle at (50, 50) with width=100, height=50, stroke_width=2
When: Calling bounds_with_stroke()
Then: Returns AABB expanded correctly from original position: (49, 49, 151, 101)

## Error Path Tests

### test_returns_error_when_min_x_greater_than_max_x
Given: min_x=100, min_y=0, max_x=50, max_y=100
When: Creating AABB::new
Then: Returns Err(BoundsError::InvalidBounds)

### test_returns_error_when_min_y_greater_than_max_y
Given: min_x=0, min_y=100, max_x=100, max_y=50
When: Creating AABB::new
Then: Returns Err(BoundsError::InvalidBounds)

### test_expand_with_zero_is_valid
Given: AABB(0, 0, 100, 100)
When: Expanding by 0
Then: Returns AABB(0, 0, 100, 100) - no change

## Edge Case Tests

### test_aabb_expand_with_very_small_amount
Given: AABB(0, 0, 100, 100)
When: Expanding by 0.001
Then: Returns AABB(-0.001, -0.001, 100.001, 100.001)

### test_aabb_expand_with_large_amount
Given: AABB(0, 0, 100, 100)
When: Expanding by 1000
Then: Returns AABB(-1000, -1000, 1100, 1100)

### test_zero_dimension_aabb
Given: AABB(0, 0, 0, 0)
When: Expanding by 10
Then: Returns AABB(-10, -10, 10, 10)

### test_stroke_width_zero
Given: Rectangle at (0, 0) with width=100, height=50, stroke_width=0
When: Calling bounds_with_stroke()
Then: Returns same AABB as without stroke: (0, 0, 100, 50)

### test_hit_margin_zero
Given: AABB(0, 0, 100, 100)
When: Adding hit margin of 0
Then: Returns same AABB: (0, 0, 100, 100)

## Contract Verification Tests

### test_precondition_min_max_validation
Given: Invalid bounds (max < min)
When: Attempting to create AABB
Then: Returns Err with specific error variant

### test_postcondition_expand_correct_amount
Given: AABB(10, 20, 110, 120), expand by 5
When: Calling expand(5)
Then: min_x=5, min_y=15, max_x=115, max_y=125

### test_postcondition_center_preservation
Given: AABB with known center (50, 50) = (0, 0, 100, 100)
When: Expanding by any amount
Then: Center remains at (50, 50)

### test_invariant_width_non_negative
Given: Any valid AABB
When: Calling width()
Then: Returns value >= 0

### test_invariant_height_non_negative
Given: Any valid AABB
When: Calling height()
Then: Returns value >= 0

## Given-When-Then Scenarios

### Scenario 1: Stroke Width Inclusion
Given: A rectangle shape with stroke
When: Computing bounds for hit testing
Then: The bounds should include half the stroke width on each side

### Scenario 2: Hit Margin for Selection
Given: A shape with a 5px hit margin configured
When: Computing bounds for click detection
Then: The bounds should be expanded by the hit margin

### Scenario 3: Combined Stroke and Hit Margin
Given: A stroked shape with hit margin
When: Computing effective bounds
Then: Both stroke width and hit margin are included in the final AABB
