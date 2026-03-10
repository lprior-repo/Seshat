# Martin Fowler Test Plan: Fit-to-Content Camera with Padding and Huge Coordinate Handling

## Happy Path Tests
- `test_fit_to_content_with_valid_bounds_and_padding_returns_transform`
  Given: Valid AABB content (0,0 to 100,100) and padding 20.0
  When: fit_to_content is called
  Then: Returns FitTransform with finite scale and offsets

- `test_fit_to_content_centers_content_in_viewport`
  Given: Content at (100,100) to (200,200), viewport 800x600
  When: fit_to_content is called
  Then: Content center aligns with viewport center

- `test_fit_to_content_respects_max_zoom`
  Given: Very small content (1x1) in large viewport (800x600), padding 0
  When: fit_to_content is called
  Then: Scale is clamped to MAX_ZOOM (4.0)

- `test_fit_to_content_respects_min_zoom`
  Given: Very large content (10000x10000) in small viewport (100x100), padding 0
  When: fit_to_content is called
  Then: Scale is clamped to MIN_ZOOM (0.1)

## Error Path Tests
- `test_fit_to_content_returns_error_when_padding_negative`
  Given: Valid content AABB, padding = -10.0
  When: fit_to_content is called
  Then: Returns Err(Error::InvalidPadding)

- `test_fit_to_content_returns_error_when_content_width_zero`
  Given: AABB with zero width (min_x=0, max_x=0), padding 10.0
  When: fit_to_content is called
  Then: Returns Err(Error::InvalidContentBounds)

- `test_fit_to_content_returns_error_when_content_height_zero`
  Given: AABB with zero height (min_y=0, max_y=0), padding 10.0
  When: fit_to_content is called
  Then: Returns Err(Error::InvalidContentBounds)

- `test_fit_to_content_returns_error_when_content_invalid`
  Given: AABB where min_x > max_x (invalid bounds)
  When: fit_to_content is called
  Then: Returns Err(Error::InvalidContentBounds)

## Edge Case Tests
- `test_fit_to_content_handles_extreme_positive_coordinates`
  Given: Content at (1e15, 1e15) to (1e15+100, 1e15+100), padding 0
  When: fit_to_content is called
  Then: Returns FitTransform with finite values (no overflow)

- `test_fit_to_content_handles_extreme_negative_coordinates`
  Given: Content at (-1e15-100, -1e15-100) to (-1e15, -1e15), padding 0
  When: fit_to_content is called
  Then: Returns FitTransform with finite values

- `test_fit_to_content_handles_mixed_extreme_coordinates`
  Given: Content spanning huge range: (-1e15, -1e15) to (1e15, 1e15)
  When: fit_to_content is called
  Then: Returns FitTransform with finite values

- `test_fit_to_content_handles_zero_padding`
  Given: Valid content, padding = 0.0
  When: fit_to_content is called
  Then: Returns FitTransform with content filling viewport

- `test_fit_to_content_handles_large_padding`
  Given: Valid content, padding = 1000.0 (larger than content)
  When: fit_to_content is called
  Then: Returns FitTransform with MIN_ZOOM (content barely visible)

- `test_fit_to_content_handles_coordinate_overflow_scenario`
  Given: Content at (1e308, 1e308) to (1e308+1e200, 1e308+1e200)
  When: fit_to_content is called
  Then: Returns Error::CoordinateOverflow or safe finite result (NOT NaN/Infinity)

## Contract Verification Tests
- `test_precondition_padding_non_negative`
- `test_precondition_content_bounds_valid`
- `test_precondition_content_dimensions_positive`
- `test_postcondition_scale_finite`
- `test_postcondition_offsets_finite`
- `test_postcondition_scale_clamped`
- `test_invariant_zoom_bounds`

## Contract Violation Tests
- `test_padding_violation_returns_invalid_padding_error`
  Given: padding = -5.0
  When: fit_to_content is called
  Then: Returns Err(Error::InvalidPadding)

- `test_invalid_bounds_violation_returns_invalid_content_error`
  Given: AABB where min > max
  When: fit_to_content is called
  Then: Returns Err(Error::InvalidContentBounds)

- `test_zero_dimension_violation_returns_invalid_content_error`
  Given: AABB with width = 0 or height = 0
  When: fit_to_content is called
  Then: Returns Err(Error::InvalidContentBounds)

## Given-When-Then Scenarios

### Scenario 1: Fit Large Diagram to Screen
Given: User has a 10000x5000 unit diagram positioned at origin
When: User triggers fit-to-content with 50px padding
Then:
- Camera zooms out to show entire diagram
- Diagram is centered in viewport
- Padding is respected around edges

### Scenario 2: Fit Extreme Coordinates Diagram
Given: User has elements at coordinates from -1e12 to +1e12
When: User triggers fit-to-content
Then:
- No float overflow occurs
- All elements visible in viewport
- Camera positioned at content center

### Scenario 3: Fit Empty/Negative Area (Error Case)
Given: Invalid content bounds (min > max or zero area)
When: User triggers fit-to-content
Then:
- Returns error (does not panic)
- No state modification
- Error message indicates invalid content
