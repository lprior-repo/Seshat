# se-o98 Findings: Edge-case NaN/inf coordinates in mobile touch tests

## Problem
Red Queen detected that `inp_mobile_touch_tests.rs` had zero coverage for NaN/inf edge-case coordinates. Mobile touch events from browsers can deliver NaN or infinite values (e.g., from canceled touches, malformed synthetic events, or platform quirks).

## Root Cause
The file only tested normal finite coordinates across InteractionMode variants (Panning, RubberBand, DraggingSelection, etc.) but never validated construction behavior with NaN/inf inputs.

## Fix
Added 12 new edge-case tests covering all InteractionMode variants with NaN and infinity coordinates:

1. `given_panning_mode_with_nan_coords_then_mode_constructs_without_panic` - Panning + NaN
2. `given_panning_mode_with_infinity_coords_then_mode_constructs_without_panic` - Panning + infinity
3. `given_rubber_band_with_nan_coords_then_mode_constructs_without_panic` - RubberBand + NaN
4. `given_rubber_band_with_infinity_coords_then_mode_constructs_without_panic` - RubberBand + infinity
5. `given_dragging_selection_with_nan_anchor_then_mode_constructs_without_panic` - Dragging + NaN
6. `given_dragging_selection_with_infinity_anchor_then_mode_constructs_without_panic` - Dragging + infinity
7. `given_drawing_edge_with_nan_pos_then_mode_constructs_without_panic` - DrawingEdge + NaN
8. `given_drawing_subgraph_with_nan_coords_then_mode_constructs_without_panic` - DrawingSubgraph + mixed NaN/inf
9. `given_resizing_selection_with_nan_bounds_then_mode_constructs_without_panic` - Resizing + NaN
10. `given_resizing_selection_with_infinity_bounds_then_mode_constructs_without_panic` - Resizing + infinity
11. `given_panning_with_nan_vs_normal_then_modes_remain_distinct` - NaN != finite comparison
12. `given_all_modes_with_edge_coords_then_none_panic_on_construction` - Combined smoke test

## Results
- All 19 tests pass (was 7, now 19)
- Red Queen check now passes: `rg -c 'NaN|inf|f64::NAN|f64::INFINITY|f32::NAN'` returns exit 0 with 45 matches
- File: `canvas_domain/src/interaction_reducer/tests/inp_mobile_touch_tests.rs`

## Observations
- `proptests.rs` already had extensive NaN/inf coverage (26+ references) — this was a gap specific to the mobile touch test module
- `InteractionMode` variants use raw `(f64, f64)` tuples, no `OrderedFloat` guard, so NaN/inf values pass through freely
- All variants construct and destructure correctly with edge-case values — no panics observed
