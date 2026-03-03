# Martin Fowler Given-When-Then Tests: Geometry Math (GEO-001 to GEO-030)

**Bead ID**: bd-2qj
**Title**: geometry: Implement geometry math tests (GEO-001 to GEO-030)

## GEO-001: AABB for Axis-Aligned Rectangles

### Test: Axis-aligned rectangle at origin
```gherkin
Given a rectangle at position (0, 0) with width 100 and height 50
When I calculate the AABB
Then the AABB should have min (0, 0) and max (100, 50)
```

### Test: Axis-aligned rectangle with offset
```gherkin
Given a rectangle at position (50, 25) with width 100 and height 50
When I calculate the AABB
Then the AABB should have min (50, 25) and max (150, 75)
```

## GEO-002: AABB for Rotated Rectangles

### Test: Square rotated 45 degrees
```gherkin
Given a square at (0, 0) with size 100 rotated 45 degrees
When I calculate the AABB
Then the AABB should be expanded by sqrt(2) factor
And the center should remain at (50, 50)
```

### Test: Rectangle rotated 90 degrees
```gherkin
Given a rectangle at (0, 0) with width 100 and height 50 rotated 90 degrees
When I calculate the AABB
Then the AABB dimensions should be swapped (50 wide, 100 tall)
```

### Test: Rectangle rotated 180 degrees
```gherkin
Given a rectangle rotated 180 degrees
When I calculate the AABB
Then the AABB should equal the unrotated bounds
```

## GEO-003: Stroke Width Inclusion

### Test: Rectangle with stroke
```gherkin
Given a rectangle at (0, 0) with width 100 and height 50 and stroke width 4
When I calculate bounds with stroke
Then the bounds should be expanded by 2 on each side
And min should be (-2, -2) and max should be (102, 52)
```

### Test: Zero stroke width
```gherkin
Given a rectangle with stroke width 0
When I calculate bounds with stroke
Then the bounds should equal the shape bounds
```

## GEO-004: Text Bounds Calculation

### Test: Basic text
```gherkin
Given text "Hello" at position (10, 20) with font size 16
When I calculate text bounds
Then the bounds should start at (10, 20)
And the height should be 16
And the width should be 48 (0.6 * 16 * 5 characters)
```

### Test: Empty text
```gherkin
Given empty text at position (10, 20)
When I calculate text bounds
Then the bounds should have zero width
And the height should equal font size
```

### Test: Emoji text
```gherkin
Given text with emoji characters
When I calculate text bounds
Then the bounds should account for Unicode character count
```

### Test: RTL text
```gherkin
Given right-to-left text
When I calculate text bounds
Then the bounds should be calculated correctly
```

## GEO-005: Image Bounds

### Test: Image with dimensions
```gherkin
Given an image at (50, 100) with width 200 and height 150
When I calculate image bounds
Then min should be (50, 100) and max should be (250, 250)
```

### Test: Image at origin
```gherkin
Given an image at (0, 0) with size 100x100
When I calculate image bounds
Then min should be (0, 0) and max should be (100, 100)
```

## GEO-006: Scale Around Anchor

### Test: Scale away from anchor
```gherkin
Given a point at (100, 100) and anchor at (50, 50)
When I scale by factor 2
Then the point should move to (150, 150)
```

### Test: Scale anchor point itself
```gherkin
Given a point at anchor position (50, 50)
When I scale around that anchor by any factor
Then the point should remain at (50, 50)
```

### Test: Scale toward anchor (shrink)
```gherkin
Given a point at (100, 100) and anchor at (50, 50)
When I scale by factor 0.5
Then the point should move to (75, 75)
```

## GEO-007: Rotate Around Center

### Test: Rotate 90 degrees
```gherkin
Given a point at (100, 0) and center at origin
When I rotate 90 degrees counter-clockwise
Then the point should be at (0, 100)
```

### Test: Rotate 180 degrees
```gherkin
Given a point at (100, 0) and center at origin
When I rotate 180 degrees
Then the point should be at (-100, 0)
```

### Test: Rotate center itself
```gherkin
Given a point at center position (50, 50)
When I rotate around that center by any angle
Then the point should remain at (50, 50)
```

### Test: Rotate 45 degrees
```gherkin
Given a point at (1, 0) and center at origin
When I rotate 45 degrees
Then the point should be at (sqrt(2)/2, sqrt(2)/2)
```

## GEO-008: Resize with Aspect Lock

### Test: Maintain 2:1 aspect ratio
```gherkin
Given original dimensions 100x50 (2:1 aspect ratio)
When I resize width to 200
Then the new height should be 100
```

### Test: Shrink with aspect lock
```gherkin
Given original dimensions 100x50
When I resize width to 50
Then the new height should be 25
```

### Test: Square aspect ratio
```gherkin
Given square dimensions 100x100
When I resize width to 200
Then the new height should be 200
```

## GEO-009: Combined Transforms

### Test: Scale then rotate
```gherkin
Given a point at (2, 0) and anchor at origin
When I scale by 2 then rotate 90 degrees
Then the point should be at (0, 4)
```

### Test: Transform order is significant
```gherkin
Given a point at (1, 0)
When I apply scale then rotate vs rotate then scale
Then both should be deterministic and consistent
```

## GEO-010: Safe Bounds Edge Cases

### Test: Zero-size bounds
```gherkin
Given bounds with min and max both at (0, 0)
When I create safe bounds
Then the result should be valid with zero dimensions
```

### Test: Negative coordinates
```gherkin
Given bounds with negative coordinates (-100, -50) to (-10, -5)
When I create safe bounds
Then the result should be valid
```

### Test: NaN coordinates
```gherkin
Given bounds with NaN value
When I create safe bounds
Then the result should be None
```

### Test: Infinity coordinates
```gherkin
Given bounds with infinity value
When I create safe bounds
Then the result should be None
```

### Test: Swapped min/max
```gherkin
Given bounds where min > max (100, 100) to (0, 0)
When I create safe bounds
Then the result should correct the order to min (0, 0) max (100, 100)
```

## GEO-012: Zoom at Pointer

### Test: Zoom at center
```gherkin
Given view center at origin and pointer at origin
When I zoom by factor 2
Then the view center should remain at origin
```

### Test: Zoom with offset pointer
```gherkin
Given view center at (100, 100) and pointer at (50, 50)
When I zoom in by factor 2
Then the view center should move to (150, 150)
```

### Test: Zoom out
```gherkin
Given view center at (100, 100) and pointer at (50, 50)
When I zoom out by factor 0.5
Then the view center should move to (75, 75)
```

## GEO-013: Snap Horizontal Lines

### Test: Snap within tolerance
```gherkin
Given a horizontal line at y=52 and snap targets [0, 50, 100]
When I snap with tolerance 5
Then the line should snap to y=50
```

### Test: No snap outside tolerance
```gherkin
Given a horizontal line at y=60 and snap targets [0, 50, 100]
When I snap with tolerance 5
Then no snap should occur
```

## GEO-014: Snap Vertical Lines

### Test: Snap within tolerance
```gherkin
Given a vertical line at x=102 and snap targets [0, 100, 200]
When I snap with tolerance 5
Then the line should snap to x=100
```

### Test: Prefer closest target
```gherkin
Given a vertical line at x=48 and snap targets [0, 100]
When I snap with tolerance 50
Then the line should snap to x=0 (closest)
```

## GEO-015: Grid Snapping

### Test: Snap to nearest grid
```gherkin
Given a point at (47, 53) and grid size 10
When I snap to grid
Then the point should snap to (50, 50)
```

### Test: Point already on grid
```gherkin
Given a point at (50, 100) on the grid
When I snap to grid
Then the point should remain at (50, 100)
```

### Test: Negative coordinates
```gherkin
Given a point at (-47, -53) and grid size 10
When I snap to grid
Then the point should snap to (-50, -50)
```

## GEO-016: Edge Routing

### Test: L-shaped route
```gherkin
Given source at (0, 0) and target at (100, 50)
When I compute orthogonal route
Then the route should have 3 points forming L-shape
```

### Test: Vertical route
```gherkin
Given vertically aligned points
When I compute orthogonal route
Then the route should be a direct line
```

## GEO-018: Fit to Viewport

### Test: Perfect fit
```gherkin
Given content 100x100 and viewport 100x100
When I compute fit transform
Then the scale should be 1.0
```

### Test: Scale down
```gherkin
Given content 200x200 and viewport 100x100
When I compute fit transform
Then the scale should be 0.5
```

### Test: With padding
```gherkin
Given content 100x100 and viewport 120x120 with padding 10
When I compute fit transform
Then the scale should account for padding
```

## GEO-019: Hit Test with Margin

### Test: Point inside rectangle
```gherkin
Given a point at (50, 50) inside rectangle (0, 0, 100, 100)
When I hit test with margin 5
Then the result should be true
```

### Test: Point within margin
```gherkin
Given a point at (-3, 50) just outside rectangle
When I hit test with margin 5
Then the result should be true
```

### Test: Point outside margin
```gherkin
Given a point at (-10, 50) outside margin
When I hit test with margin 5
Then the result should be false
```

## GEO-020: Hit Test Rotated

### Test: Hit rotated rectangle center
```gherkin
Given a 45-degree rotated square
When I hit test the center point
Then the result should be true
```

### Test: Miss rotated rectangle
```gherkin
Given a rotated square and point far away
When I hit test
Then the result should be false
```

## GEO-021: World-Screen Round-Trip

### Test: Round-trip preserves position
```gherkin
Given a world point (100, 200), camera (50, 75), and zoom 2
When I transform to screen and back to world
Then the result should equal the original point
```

### Test: Round-trip at origin
```gherkin
Given world point at origin
When I transform to screen and back
Then the result should be origin
```

## GEO-025: Rotation Drift

### Test: Repeated tiny rotations
```gherkin
Given 1000 tiny rotations summing to ~57 degrees
When I compare with single rotation of total angle
Then the drift should be < 1e-6
```

### Test: Full circle drift
```gherkin
Given 1000 tiny rotations for full 360 degrees
When I compare final position with start
Then the drift should be < 1e-6
```

## GEO-027-028: Camera Constraints

### Test: Min zoom clamp
```gherkin
Given zoom value below minimum (0.1)
When I clamp zoom
Then the result should be 0.1
```

### Test: Max zoom clamp
```gherkin
Given zoom value above maximum (10.0)
When I clamp zoom
Then the result should be 10.0
```

## GEO-029: Pan with Zoom

### Test: Pan scales with zoom
```gherkin
Given screen delta of 10 pixels
When I convert to world delta at various zoom levels
Then world delta should be inversely proportional to zoom
```

## GEO-030: Extreme Coordinates

### Test: Extreme world coordinates
```gherkin
Given world coordinates at +/- 1e6
When I transform to screen
Then screen coordinates should be finite
```

### Test: Round-trip at extremes
```gherkin
Given extreme world coordinates
When I round-trip through screen space
Then the relative error should be < 1e-10
```

## Property-Based Tests

### Scale anchor invariance
```gherkin
Given any scale factor
When I scale the anchor point around itself
Then the anchor should remain unchanged
```

### Rotation center invariance
```gherkin
Given any rotation angle
When I rotate the center point around itself
Then the center should remain unchanged
```

### Full circle invariance
```gherkin
Given any rotation angle
When I rotate by angle then by (2PI - angle)
Then the point should return to original position
```

### AABB containment
```gherkin
Given any rectangle with any rotation
When I calculate AABB
Then all corners should be within the AABB
```

### Aspect ratio preservation
```gherkin
Given any valid dimensions
When I resize with aspect lock
Then the aspect ratio should be preserved
```
