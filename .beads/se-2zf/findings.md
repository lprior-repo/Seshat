# REDQUEEN Findings: canvas_domain/src/interaction_reducer/types.rs

## Summary

Co-evolutionary adversarial testing against `types.rs` (90 lines). Added 36 new tests to `red_queen_types.rs`, bringing total from 34 to 70. All pass.

## Bugs Found

### 1. Tautology Test (CRITICAL - existing test bug)
- **File**: `red_queen_types.rs:277-283` (`rq_types_resize_handle_ord_not_implemented`)
- **Issue**: The test declares `fn requires_ord<T: Ord>() {}` but **never calls it with `ResizeHandle`**. It then returns `false` from a separate function. This test passes regardless of whether `ResizeHandle` implements `Ord`. It provides ZERO verification.
- **Status**: Not fixed (existing test, not implementation)

### 2. ResizeHandle Missing Hash Derive (MEDIUM)
- **File**: `types.rs:80`
- **Issue**: `ResizeHandle` derives `Clone, Copy, Debug, PartialEq, Eq` but NOT `Hash`. If ever used as a `HashMap` key or `HashSet` element, compilation will fail.
- **Fix**: Add `#[derive(Hash)]` to `ResizeHandle`

## Design Weaknesses

### 3. No Validation on aspect_ratio Field (HIGH)
- **File**: `types.rs:69` (`ResizingSelection { aspect_ratio: Option<f64> }`)
- **Issue**: The field accepts `Some(0.0)` (division-by-zero risk), `Some(-1.5)` (negative ratio), `Some(f64::NAN)`, and `Some(f64::INFINITY)` without any validation. Downstream consumers in `resize.rs` and `release.rs` would need to guard against these.
- **Recommendation**: Either validate at construction or use a newtype wrapper

### 4. No Validation on NodeId/EdgeId Fields (LOW)
- Empty string `NodeId`/`EdgeId` accepted in `DrawingEdge.from_node` and `DraggingBendPoint.edge_id`

### 5. No Validation on Bounds (LOW)
- `ResizingSelection.original_bounds` accepts negative and zero-width/height tuples
- `RubberBand` allows `start > current` (inverted selection)

### 6. NaN Breaks PartialEq on ALL Variants with f64 (MEDIUM)
- `InteractionMode` derives `PartialEq` but contains f64 fields in 6 of 8 variants. NaN in any f64 field makes equality comparisons return false even for structurally identical values. This is a known Rust footgun documented for `Panning` in the existing tests, but the same issue affects:
  - `RubberBand` (start, current)
  - `DraggingSelection` (anchor_canvas, anchor_client)
  - `DrawingEdge` (current_pos)
  - `DrawingSubgraph` (start, current)
  - `ResizingSelection` (original_bounds, anchor, aspect_ratio)
- **Recommendation**: Consider using `ordered_float::NotNan<f64>` for position fields

## Test Coverage Gaps Fixed

The existing suite had these gaps, all now covered:

1. Only 4 of 8 InteractionMode variants tested for pairwise inequality → now all 8
2. `CommitError::UpdateFailed` had no equality/clone/debug tests → added
3. `LabelEditError::TargetNotFound → CommitError` conversion untested → added
4. NaN propagation tested only for `Panning` → now tested for all 6 variants with f64 fields
5. `did_move=true` vs `did_move=false` not tested → added
6. `did_resize=true` vs `did_resize=false` not tested → added
7. Debug format tested only for `Select` and `Panning` → now all 8 variants
8. Self-equality not verified across all variants → added
9. Subnormal float handling not tested → added
10. No negative/zero aspect ratio tests → added

## Tests Added (36)

- `rq_types_all_eight_interaction_mode_variants_pairwise_distinct`
- `rq_types_commit_error_update_failed_equality`
- `rq_types_commit_error_update_failed_not_equal_to_label_edit`
- `rq_types_commit_error_update_failed_clone_roundtrip`
- `rq_types_from_target_not_found_to_commit_error`
- `rq_types_commit_error_update_failed_debug`
- `rq_types_rubberband_nan_breaks_equality`
- `rq_types_rubberband_nan_current_breaks_equality`
- `rq_types_drawing_subgraph_nan_breaks_equality`
- `rq_types_resizing_nan_original_bounds_breaks_equality`
- `rq_types_resizing_nan_anchor_breaks_equality`
- `rq_types_dragging_selection_nan_anchor_breaks_equality`
- `rq_types_drawing_edge_nan_pos_breaks_equality`
- `rq_types_dragging_selection_did_move_true_distinct_from_false`
- `rq_types_resizing_did_resize_true_distinct_from_false`
- `rq_types_resizing_negative_aspect_ratio_accepted`
- `rq_types_resizing_zero_aspect_ratio_accepted`
- `rq_types_resizing_degenerate_zero_bounds`
- `rq_types_resizing_negative_bounds`
- `rq_types_rubberband_inverted_selection`
- `rq_types_rubberband_zero_area`
- `rq_types_drawing_edge_without_port`
- `rq_types_drawing_edge_empty_node_id`
- `rq_types_dragging_bend_point_empty_edge_id`
- `rq_types_resize_handle_all_variants_exhaustive_match`
- `rq_types_interaction_mode_debug_all_variants`
- `rq_types_interaction_mode_self_equality_all_variants`
- `rq_types_dragging_selection_with_populated_positions`
- `rq_types_resizing_with_populated_originals`
- `rq_types_subnormal_floats_preserved`
- `rq_types_resize_handle_is_copy_and_eq`
- `rq_types_resize_handle_is_not_hash`
- `rq_types_label_edit_error_both_variants_clone_roundtrip`
- `rq_types_label_edit_error_not_equal`
- `rq_types_commit_error_label_edit_variants_not_equal`
- `rq_types_dispatch_error_clone_eq`
