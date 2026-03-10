# Martin Fowler Test Plan

## Happy Path Tests

### test_container_resize_with_scale_children_mode
Given: A container with 2 children at positions (10,10) and (50,50), widths 20 each
When: Resize container to width=200, height=200 with ScaleChildren mode
Then:
- Container dimensions are 200x200
- First child position scales from (10,10) to (50,50) (5x scale)
- Second child position scales from (50,50) to (250,250) (5x scale)
- Children widths scale from 20 to 100

### test_container_resize_with_expand_container_mode
Given: A container at (0,0) with width=100, height=100 containing a child at (80,80) with width=30, height=30
When: Resize with ExpandContainer mode to fit child with padding 10
Then:
- Container expands to width=120, height=120
- Child remains at original position (80,80)
- Child dimensions unchanged

### test_nested_container_resize
Given: Outer container containing inner container, both with children
When: Resize outer container with ScaleChildren
Then:
- Outer container dimensions updated
- Inner container and its children all scale proportionally

## Error Path Tests

### test_returns_error_when_container_not_found
Given: State with nodes but no container
When: apply_container_resize with container_id="nonexistent"
Then: Returns Err(ResizeError::ContainerNotFound)

### test_returns_error_when_invalid_dimensions_negative
Given: Valid container
When: apply_container_resize with width=-50, height=100
Then: Returns Err(ResizeError::InvalidDimensions)

### test_returns_error_when_invalid_dimensions_zero
Given: Valid container
When: apply_container_resize with width=0, height=100
Then: Returns Err(ResizeError::InvalidDimensions)

### test_returns_error_when_container_too_small_expand_mode
Given: Container with child at (90,90) width=30, height=30
When: Resize to width=50, height=50 with ExpandContainer mode
Then: Returns Err(ResizeError::ContainerTooSmall)

## Edge Case Tests

### test_resize_empty_container
Given: Container with no children
When: Resize to any dimensions
Then: Succeeds, container dimensions updated

### test_resize_container_single_child
Given: Container with single child centered
When: Resize with ScaleChildren
Then: Child scales proportionally, stays centered

### test_resize_preserves_child_z_index
Given: Container with children at different z_index values
When: Resize container
Then: z_index values preserved for all children

### test_resize_preserves_child_order
Given: Container with children in specific order
When: Resize container
Then: Child order in node map preserved

### test_resize_maintains_relative_positions
Given: Container with child at 25% width, 25% height position
When: Resize with ScaleChildren (2x)
Then: Child at 50% width, 50% height (relative position preserved)

## Contract Verification Tests

### test_precondition_container_exists
Given: State
When: Call with non-existent container ID
Then: Err(ResizeError::ContainerNotFound)

### test_precondition_dimensions_valid
Given: State with container
When: Call with invalid dimensions
Then: Err(ResizeError::InvalidDimensions)

### test_postcondition_container_updated
Given: State with container at 100x100
When: Resize to 200x200
Then: Container in result state has 200x200

### test_postcondition_children_transformed
Given: Container with child
When: Resize with ScaleChildren
Then: Child transformed proportionally

### test_invariant_container_encompasses_children
Given: Container with children
When: Any resize operation
Then: Container bounds always encompass all children

### test_invariant_children_within_container
Given: Container with children
When: After resize
Then: All children positions within container bounds

## Given-When-Then Scenarios

### Scenario 1: Scale Children Mode Preserves Relative Layout
Given: Container at (0,0) 100x100 with child at (25,25) size 50x50
When: Resize to 200x200 with ScaleChildren
Then:
- Container is 200x200
- Child is at (50,50) size 100x100
- Child remains at 25% position within container

### Scenario 2: Expand Container Keeps Children Fixed
Given: Container at (0,0) 100x100 with child at (80,80) size 30x30
When: Resize with ExpandContainer and child requires 120x120
Then:
- Container expands to accommodate child with padding
- Child remains at absolute position (80,80)
- Child size unchanged

### Scenario 3: Nested Containers All Scale
Given: Outer container (0,0) 200x200 containing inner container (50,50) 100x100 containing leaf child
When: Resize outer to 400x400 with ScaleChildren
Then:
- Outer: 400x400
- Inner: scales to 200x200 at position (100,100)
- Leaf: scales proportionally
