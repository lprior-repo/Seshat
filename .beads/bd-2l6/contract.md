bead_id: bd-2l6
bead_title: tests: Implement GEO geometry tests (GEO-001 to GEO-010)
phase: p0
updated_at: 2026-03-01T21:30:00Z

# Contract: GEO Geometry Tests (GEO-001 to GEO-010)

## Purpose

Implement 10 geometry tests covering core geometry operations for the diagram tool.

## Scope

Create a new geometry module (`diagram_tool/src/geometry/mod.rs`) with comprehensive tests for:

### GEO-001: AABB for Axis-Aligned Rectangles
- Test that axis-aligned bounding box calculation returns correct bounds
- Input: rectangle at origin with width/height
- Output: AABB equal to the rectangle itself

### GEO-002: AABB for Rotated Rectangles
- Test AABB calculation for rectangles rotated at various angles
- Rotation expands the bounding box
- 45-degree rotation of square should expand bounds by sqrt(2)/2 factor

### GEO-003: Stroke Width Inclusion in Bounds
- Test that stroke width is included in bounds calculation
- Stroke extends beyond the shape boundary by stroke_width/2 on each side
- Total bounds expansion = stroke_width

### GEO-004: Text Bounds Calculation
- Test text bounding box calculation based on font metrics
- Bounds should include text width and height based on font size
- Text at position (x, y) should have bounds starting at that position

### GEO-005: Image Bounds Calculation
- Test image bounds based on position and dimensions
- Image bounds = position + width/height
- Bounds should be deterministic for same inputs

### GEO-006: Scale Around Anchor Point
- Test scaling operation that keeps an anchor point fixed
- When scaling by factor k around anchor (ax, ay):
  - New x = ax + (x - ax) * k
  - New y = ay + (y - ay) * k

### GEO-007: Rotate Around Center
- Test rotation of points around a center point
- Rotation by angle theta around center (cx, cy):
  - New x = cx + (x - cx) * cos(theta) - (y - cy) * sin(theta)
  - New y = cy + (x - cx) * sin(theta) + (y - cy) * cos(theta)

### GEO-008: Resize with Aspect Ratio Lock
- Test resize operation that maintains aspect ratio
- When resizing width, height should adjust proportionally
- Original aspect ratio = width / height must be preserved

### GEO-009: Combined Transform Chain
- Test that multiple transforms compose correctly
- Scale followed by rotate should produce correct final position
- Transform order matters and should be deterministic

### GEO-010: Bounds Edge Cases
- Test bounds calculation with edge cases:
  - Zero-sized shapes
  - Negative coordinates
  - Very large coordinates
  - Degenerate inputs

## Preconditions

1. Project must compile (`moon run :quick` passes)
2. Existing test patterns in `diagram_tool/src/export/svg.rs` can be used as reference
3. No unsafe code allowed
4. All tests must use `#[test]` attribute and follow existing patterns

## Postconditions

1. All 10 GEO tests pass (`moon run :test` passes)
2. New geometry module exists at `diagram_tool/src/geometry/mod.rs`
3. Module is properly integrated into the crate
4. Each test has clear given/when/then structure
5. Tests use property-based testing (proptest) where appropriate

## Acceptance Criteria

- [ ] GEO-001: AABB axis-aligned test passes
- [ ] GEO-002: AABB rotated test passes
- [ ] GEO-003: Stroke width inclusion test passes
- [ ] GEO-004: Text bounds test passes
- [ ] GEO-005: Image bounds test passes
- [ ] GEO-006: Scale around anchor test passes
- [ ] GEO-007: Rotate around center test passes
- [ ] GEO-008: Aspect ratio lock test passes
- [ ] GEO-009: Combined transform test passes
- [ ] GEO-010: Edge cases test passes
- [ ] All tests pass with `moon run :test`
- [ ] CI passes with `moon run :ci`

## Test Mapping

| Test ID | Test Function Name | Category |
|---------|-------------------|----------|
| GEO-001 | test_aabb_axis_aligned | bounds |
| GEO-002 | test_aabb_rotated_rectangle | bounds |
| GEO-003 | test_stroke_width_inclusion | bounds |
| GEO-004 | test_text_bounds | bounds |
| GEO-005 | test_image_bounds | bounds |
| GEO-006 | test_scale_around_anchor | transform |
| GEO-007 | test_rotate_around_center | transform |
| GEO-008 | test_resize_aspect_lock | transform |
| GEO-009 | test_combined_transforms | transform |
| GEO-010 | test_bounds_edge_cases | edge-cases |
