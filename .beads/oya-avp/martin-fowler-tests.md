# Martin Fowler Test Plan: Aspect Lock During Multi-Select Resize (MUL-013)

## Happy Path Tests
- test_resize_selection_without_aspect_lock_uses_original_behavior
  - Given: Multiple nodes selected with no aspect lock
  - When: Resizing from SE handle by delta (10, 10)
  - Then: Selection bounds expand freely without ratio constraint

- test_resize_selection_with_aspect_lock_preserves_ratio
  - Given: Multiple nodes selected with aspect_ratio = Some(2.0)
  - When: Resizing from SE handle changes width by 20 units
  - Then: Height changes by 10 units (maintaining 2:1 ratio)

- test_aspect_ratio_calculated_from_initial_bounds
  - Given: Selection bounds are (0, 0, 100, 50) - 2:1 ratio
  - When: Resize begins with aspect lock enabled
  - Then: aspect_ratio field is set to Some(2.0)

## Error Path Tests
- test_resize_with_zero_initial_width_handles_gracefully
  - Given: Selection with zero width
  - When: Aspect ratio lock is attempted
  - Then: aspect_ratio is set to None (no lock applied)

- test_resize_with_zero_initial_height_handles_gracefully
  - Given: Selection with zero height
  - When: Aspect ratio lock is attempted
  - Then: aspect_ratio is set to None (no lock applied)

## Edge Case Tests
- test_aspect_lock_with_single_node_selection
  - Given: Single node selected with aspect_ratio = Some(1.0)
  - When: Resizing from corner
  - Then: Node scales proportionally maintaining square shape

- test_aspect_lock_disabled_by_default
  - Given: New ResizingSelection state
  - When: State is created
  - Then: aspect_ratio is None

- test_aspect_ratio_maintained_across_multiple_resize_steps
  - Given: aspect_ratio = Some(1.5), current bounds (0, 0, 150, 100)
  - When: Resize changes width by additional 30 units
  - Then: Height changes by 20 units, ratio stays 1.5

## Contract Verification Tests
- test_postcondition_aspect_ratio_preserved
  - Given: aspect_ratio = Some(r), new width w
  - When: Height is calculated
  - Then: abs(new_height - w / r) < 1e-9

- test_postcondition_all_nodes_proportionally_scaled
  - Given: Two nodes at positions (0,0) and (100,0) with same size
  - When: Resize with aspect lock
  - Then: Both nodes scale by same factor

## Given-When-Then Scenarios

### Scenario 1: Multi-select resize with aspect lock enabled
**Given**: Three nodes selected forming a bounding box of 200x100 (2:1 ratio)  
**And**: User presses Shift to enable aspect lock  
**When**: User drags SE handle 50 pixels right  
**Then**:  
- Selection height grows by 25 pixels (maintaining 2:1)  
- All three nodes scale proportionally  
- aspect_ratio field in state is Some(2.0)

### Scenario 2: Multi-select resize with aspect lock disabled
**Given**: Three nodes selected  
**And**: No modifier key pressed  
**When**: User drags SE handle 50 pixels right and 30 pixels down  
**Then**:  
- Selection grows to (new_width, new_height) freely  
- aspect_ratio field in state is None  

### Scenario 3: Toggle aspect lock mid-resize
**Given**: Resize in progress without aspect lock (aspect_ratio = None)  
**When**: User presses Shift during drag  
**Then**:  
- aspect_ratio is calculated from current bounds  
- Subsequent resize movements respect the ratio  

### Scenario 4: Resize from different handles with aspect lock
**Given**: Selection with aspect_ratio = Some(1.0) (square)  
**When**: Resize from NW handle (opposite corner)  
**Then**:  
- Width and height change in opposite direction  
- Square ratio maintained throughout
