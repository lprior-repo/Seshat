# Implementation: bd-31x - selection: close remaining hit-test regressions

## Summary
Fixed the hit-test radius calculation in `find_edge_at` to be screen-consistent across zoom levels, resolving the regression where thin edges were not detected at low zoom.

## Changes Made

### File: `diagram_tool/src/ui/canvas/canvas_view.rs`

**Line 413-419** - Modified `find_edge_at` function:

```rust
// Before (buggy - fixed world radius):
let hit_radius_world = 8.0;
let endpoint_hit_radius_world = 10.0;

// After (fixed - screen-consistent radius):
let zoom = doc.editor_state.zoom.0;
let screen_hit_radius = 17.0;
let hit_radius_world = screen_hit_radius / zoom;
let endpoint_hit_radius_world = 21.0 / zoom;
```

## Root Cause
The original implementation used a fixed 8.0 world-unit hit radius, which resulted in inconsistent screen-pixel hit detection:
- At 0.5x zoom: 8 world units = 4 screen pixels (too small to detect edges)
- At 2.0x zoom: 8 world units = 16 screen pixels (overly generous)

## Fix
The fix divides the screen-consistent hit radius by zoom to get the world-space threshold:
- `world_radius = screen_radius / zoom`

This ensures the hit radius in screen pixels remains constant (17px for edge hit, 21px for endpoint hit) regardless of zoom level.

## Test Results
All unit tests pass (489 tests):
- `given_low_zoom_when_clicking_near_edge_then_hit_test_uses_screen_consistent_radius` ✓
- `given_high_zoom_when_clicking_same_world_distance_then_hit_test_is_tighter` ✓
- `given_overlapping_edges_when_hit_distance_ties_then_selection_is_stable_by_edge_id` ✓
- `given_click_near_arrow_endpoint_when_within_endpoint_radius_then_edge_is_hit` ✓

## Contract Clauses Addressed
- **ubiquitous**: Deterministic selection results maintained (same tie-break ordering via `min_by` with edge ID comparison)
- **unwanted**: Thin edges at valid tolerance now detected at low zoom (screen-consistent radius)
