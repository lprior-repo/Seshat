# Martin Fowler Tests: Container Bounds Recomputation

## Test Strategy
Following Martin Fowler's **Given-When-Then** pattern for Behavior-Driven Development (BDD).

## Test Suite: compute_subgraph_bounds

### Test 1: Single Child Container
**Given** a subgraph container with one child node at (100, 100) with size (50, 30)  
**When** computing subgraph bounds  
**Then** result should be Some((100, 100, 50, 30))

### Test 2: Multiple Children - Horizontal Spread
**Given** a subgraph container with children at:
- Child1: (0, 0), size (50, 50)
- Child2: (100, 0), size (50, 50)
- Child3: (200, 0), size (50, 50)
**When** computing subgraph bounds  
**Then** result should be Some((0, 0, 250, 50))

### Test 3: Multiple Children - Vertical Spread
**Given** a subgraph container with children at:
- Child1: (0, 0), size (50, 50)
- Child2: (0, 100), size (50, 50)
- Child3: (0, 200), size (50, 50)
**When** computing subgraph bounds  
**Then** result should be Some((0, 0, 50, 250))

### Test 4: Multiple Children - 2D Grid
**Given** a subgraph container with children at:
- Child1: (10, 10), size (40, 30)
- Child2: (60, 20), size (30, 40)
- Child3: (20, 60), size (50, 25)
**When** computing subgraph bounds  
**Then** result should be Some((10, 10, 80, 75))

### Test 5: Empty Container
**Given** a subgraph container with no children  
**When** computing subgraph bounds  
**Then** result should be None

### Test 6: Child with Negative Coordinates
**Given** a subgraph container with children at:
- Child1: (-50, -50), size (50, 50)
- Child2: (0, 0), size (50, 50)
**When** computing subgraph bounds  
**Then** result should be Some((-50, -50, 100, 100))

### Test 7: Child Overlap
**Given** a subgraph container with overlapping children at:
- Child1: (0, 0), size (100, 100)
- Child2: (50, 50), size (100, 100)
**When** computing subgraph bounds  
**Then** result should be Some((0, 0, 150, 150))

### Test 8: Invalid Child Bounds (NaN)
**Given** a subgraph container with one valid child and one child with NaN coordinates  
**When** computing subgraph bounds  
**Then** result should contain only valid children and return valid bounds

### Test 9: Non-Container Node
**Given** a regular node (not a subgraph) with children field empty  
**When** computing subgraph bounds  
**Then** result should be None (non-containers don't compute bounds from children)

## Test Suite: Transform Integration

### Test 10: Move Child Updates Container
**Given** a subgraph with child at (0, 0) size (50, 50)  
**When** child is moved to (100, 100)  
**Then** container bounds should be recomputed to (100, 100, 50, 50)

### Test 11: Resize Child Updates Container
**Given** a subgraph with child at (0, 0) size (50, 50)  
**When** child is resized to (100, 100)  
**Then** container bounds should be recomputed to (0, 0, 100, 100)

### Test 12: Multiple Children One Moves
**Given** a subgraph with children at (0, 0) size (50, 50) and (100, 0) size (50, 50)  
**When** first child is moved to (0, 100)  
**Then** container bounds should encompass both children

### Test 13: Add Child to Container
**Given** a subgraph with one child at (0, 0) size (50, 50)  
**When** second child added at (200, 200) size (30, 30)  
**Then** container bounds should expand to include new child

### Test 14: Remove Child from Container
**Given** a subgraph with children at (0, 0) size (50, 50) and (100, 0) size (50, 50)  
**When** second child is removed  
**Then** container bounds should shrink to encompass remaining child

## Test Suite: Edge Cases

### Test 15: Single Point Child
**Given** a subgraph with, 50) child at (50 with zero size (0, 0)  
**When** computing bounds  
**Then** result should be Some((50, 50, 0, 0)) - degenerate but valid

### Test 16: All Children at Same Position
**Given** a subgraph with multiple children all at (50, 50) with varying sizes  
**When** computing bounds  
**Then** result should be the union of all overlapping bounds
