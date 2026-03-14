bead_id: seshat-3jf
bead_title: GEO-021 to GEO-025: Intersection algorithms
phase: STATE_1
updated_at: 2026-03-14T18:46:00Z

# Contract Synthesis Request

## Bead Details
- **Bead ID**: seshat-3jf
- **Title**: GEO-021 to GEO-025: Intersection algorithms
- **Description**: Implement line-line and line-rect intersections for connector logic.

## Context
This is for the geometry math layer in diagram_tool. The existing codebase has:
- Point (x: f64, y: f64) in primitives.rs
- AABB (min_x, min_y, max_x, max_y) with intersects() method for AABB-AABB
- Rectangle (x, y, width, height, rotation)
- Existing routing.rs has `segment_intersects_aabb` but only handles axis-aligned segments (horizontal/vertical)

## Requirements
Implement:
1. **GEO-021**: Line-Line intersection (boolean) - check if two line segments intersect
2. **GEO-022**: Line-Line intersection (point) - find the intersection point if it exists
3. **GEO-023**: Line-Rect intersection (boolean) - check if line segment intersects rectangle
4. **GEO-024**: Line-Rect intersection (points) - find intersection points with rectangle edges
5. **GEO-025**: Container bounds recomputation (may already exist)

## Deliverables
Write to:
- `../seshat-3jf/.beads/seshat-3jf/contract.md`
- `../seshat-3jf/.beads/seshat-3jf/martin-fowler-tests.md`

