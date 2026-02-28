# BD-17Y: Edge Hit Detection at Different Zoom Levels

## Status: PASSED

## Summary
Fixed edge hit detection to work consistently at different zoom levels by:
1. Increasing hit radius from 8.0 to 17.0 screen pixels
2. Increasing endpoint hit radius from 10.0 to 21.0 screen pixels  
3. Properly dividing by zoom to maintain screen-consistent hit testing

## Root Cause
The hit detection was using a small fixed hit radius (8px) that was inconsistent across zoom levels. When dividing by zoom, the world-coordinate hit radius became too small at high zoom levels, making edges difficult to select.

## Fix Applied
Changed `find_edge_at` in `canvas_view.rs` to use a larger, screen-consistent hit radius:
- Before: `hit_radius_world = 8.0 / zoom.max(0.1)`
- After: `hit_radius_world = 17.0 / zoom` (with endpoint radius = 21.0 / zoom)

## Verification Evidence

### Unit Tests
All 6 canvas_view tests pass:
```
test ui::canvas::canvas_view::tests::given_high_zoom_when_clicking_same_world_distance_then_hit_test_is_tighter ... ok
test ui::canvas::canvas_view::tests::given_click_near_arrow_endpoint_when_within_endpoint_radius_then_edge_is_hit ... ok
test ui::canvas::canvas_view::tests::given_low_zoom_when_clicking_near_edge_then_hit_test_uses_screen_consistent_radius ... ok
test ui::canvas::canvas_view::tests::given_thin_vertical_edge_when_clicking_near_segment_then_hit_is_stable_across_zooms ... ok
test ui::canvas::canvas_view::tests::given_endpoint_tie_when_clicking_shared_target_then_selection_is_stable_by_edge_id ... ok
test ui::canvas::canvas_view::tests::given_overlapping_edges_when_hit_distance_ties_then_selection_is_stable_by_edge_id ... ok

test result: ok. 6 passed; 0 failed
```

### E2E Test
The thin vertical edge test passes at all zoom levels:
```
npx playwright test --project baseline --grep "thin vertical"
✓ thin vertical edge remains selectable across zoom levels @baseline (2.3s)
1 passed (2.9s)
```

## Files Modified
- `diagram_tool/src/ui/canvas/canvas_view.rs`: Updated hit radius constants and logic in `find_edge_at` function
