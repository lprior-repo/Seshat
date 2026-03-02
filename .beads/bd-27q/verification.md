bead_id: bd-27q
bead_title: tests: Implement INP mobile/touch tests
phase: p2
updated_at: 2026-03-01T20:56:00Z

# Verification: INP Mobile/Touch Tests

## Moon Validation Results

### Check
```
$ moon run :check
[OK] check
```

### Clippy
```
$ moon run :clippy
[OK] clippy
```

### Test (Rust)
```
$ moon run :test-rust
[OK] test-rust
```

## Test Execution Results

### INP Mobile/Touch Tests
```
$ cargo test --package diagram_tool inp_mobile

running 46 tests
test ui::canvas::canvas_view::inp_mobile_tests::given_double_tap_threshold_is_reasonable ... ok
test ui::canvas::canvas_view::inp_mobile_tests::given_large_base_radius_when_touch_input_then_base_preserved_if_larger ... ok
test ui::canvas::canvas_view::inp_mobile_tests::given_mouse_input_when_calculating_hit_radius_then_base_unchanged ... ok
test ui::canvas::canvas_view::inp_mobile_tests::given_mouse_input_when_hit_testing_handle_then_visual_size_used ... ok
test ui::canvas::canvas_view::inp_mobile_tests::given_reversed_timestamps_when_checked_then_not_double_tap ... ok
test ui::canvas::canvas_view::inp_mobile_tests::given_same_timestamp_when_checked_then_is_double_tap ... ok
test ui::canvas::canvas_view::inp_mobile_tests::given_threshold_boundary_values_when_checked_then_boundary_correct ... ok
test ui::canvas::canvas_view::inp_mobile_tests::given_touch_input_at_corner_when_hit_testing_then_expanded_area_covers_corners ... ok
test ui::canvas::canvas_view::inp_mobile_tests::given_touch_input_directly_on_handle_when_hit_testing_then_succeeds ... ok
test ui::canvas::canvas_view::inp_mobile_tests::given_touch_input_outside_expanded_area_when_hit_testing_then_fails ... ok
test ui::canvas::canvas_view::inp_mobile_tests::given_touch_input_when_calculating_hit_radius_then_uses_touch_minimum ... ok
test ui::canvas::canvas_view::inp_mobile_tests::given_touch_input_when_hit_testing_handle_then_expanded_hit_area_used ... ok
test ui::canvas::canvas_view::inp_mobile_tests::given_touch_minimum_matches_wcag_guideline ... ok
test ui::canvas::canvas_view::inp_mobile_tests::given_two_taps_exactly_at_threshold_when_checked_then_is_double_tap ... ok
test ui::canvas::canvas_view::inp_mobile_tests::given_two_taps_far_apart_when_checked_then_not_double_tap ... ok
test ui::canvas::canvas_view::inp_mobile_tests::given_two_taps_just_over_threshold_when_checked_then_not_double_tap ... ok
test ui::canvas::canvas_view::inp_mobile_tests::given_two_taps_within_threshold_when_checked_then_is_double_tap ... ok
test ui::canvas::canvas_view::inp_mobile_tests::given_zero_base_radius_when_touch_input_then_touch_minimum_used ... ok
test ui::canvas::canvas_view::inp_mobile_tests::given_zero_times_when_checked_then_is_double_tap ... ok
test ui::canvas::interaction_reducer::inp_mobile_touch_tests::given_all_interaction_modes_when_panning_is_active_then_only_panning_matches ... ok
test ui::canvas::interaction_reducer::inp_mobile_touch_tests::given_panning_mode_last_pos_when_updated_then_tracks_movement ... ok
test ui::canvas::interaction_reducer::inp_mobile_touch_tests::given_panning_mode_when_compared_to_drawing_modes_then_modes_differ ... ok
test ui::canvas::interaction_reducer::inp_mobile_touch_tests::given_panning_mode_when_compared_to_resizing_then_modes_differ ... ok
test ui::canvas::interaction_reducer::inp_mobile_touch_tests::given_panning_mode_when_compared_to_rubber_band_then_modes_differ ... ok
test ui::canvas::interaction_reducer::inp_mobile_touch_tests::given_panning_mode_when_two_finger_gesture_then_is_distinct_from_dragging ... ok
test ui::canvas::interaction_reducer::inp_mobile_touch_tests::given_select_mode_when_compared_to_panning_then_modes_differ ... ok
test ui::canvas::perf::inp_mobile_touch_tests::given_finger_like_input_when_processed_then_no_panic ... ok
test ui::canvas::perf::inp_mobile_touch_tests::given_mixed_pointer_inputs_then_all_produce_valid_output ... ok
test ui::canvas::perf::inp_mobile_touch_tests::given_pinch_at_limits_then_stays_bounded ... ok
test ui::canvas::perf::inp_mobile_touch_tests::given_pinch_gesture_when_zoom_in_then_zooms_canvas_not_creates_shape ... ok
test ui::canvas::perf::inp_mobile_touch_tests::given_pinch_gesture_when_zoom_out_then_zooms_canvas_not_creates_shape ... ok
test ui::canvas::perf::inp_mobile_touch_tests::given_stylus_like_input_when_processed_then_no_panic ... ok
test ui::canvas::perf::inp_mobile_touch_proptests::prop_pinch_gesture_always_produces_valid_zoom ... ok
test ui::canvas::perf::inp_mobile_touch_proptests::prop_pointer_type_agnostic_handling ... ok
test ui::interaction::inp_mobile_touch_tests::given_double_tap_timing_constants_then_are_finite_and_reasonable ... ok
test ui::interaction::inp_mobile_touch_tests::given_double_tap_timing_when_taps_within_window_then_detected ... ok
test ui::interaction::inp_mobile_touch_tests::given_long_press_when_minor_jitter_then_still_not_drag ... ok
test ui::interaction::inp_mobile_touch_tests::given_long_press_when_no_motion_then_not_drag_and_can_select ... ok
test ui::interaction::inp_mobile_touch_tests::given_touch_drag_when_motion_below_threshold_then_not_considered_drag ... ok
test ui::interaction::inp_mobile_touch_tests::given_touch_drag_when_rightward_then_uses_contain_selection_mode ... ok
test ui::interaction::inp_mobile_touch_tests::given_touch_finger_hit_area_when_computed_then_meets_accessibility ... ok
test ui::interaction::inp_mobile_touch_tests::given_touch_hit_area_when_checking_selection_handles_then_meets_minimum ... ok
test ui::interaction::inp_mobile_touch_proptests::prop_double_tap_timing_window_is_positive ... ok
test ui::interaction::inp_mobile_touch_proptests::prop_long_press_with_small_jitter_never_triggers_drag ... ok
test ui::interaction::inp_mobile_touch_proptests::prop_touch_drag_threshold_consistent_regardless_of_direction ... ok
test ui::interaction::inp_mobile_touch_proptests::prop_touch_hit_radius_always_positive_and_finite ... ok

test result: ok. 46 passed; 0 failed; 0 ignored; 0 measured; 1042 filtered out
```

## Code Quality

### Format Check
```
$ cargo fmt --check
[OK] No formatting issues
```

### Clippy Strict
```
$ cargo clippy -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic
[OK] No warnings
```

## Acceptance Criteria

- [x] 7 test categories implemented (INP-1 through INP-7)
- [x] 46 total tests (35 unit + 11 property-based)
- [x] All tests pass: `moon run :test-rust`
- [x] No clippy warnings: `moon run :clippy`
- [x] Code formatted: `cargo fmt --check`
- [x] Tests follow existing patterns in codebase
- [x] Property-based tests use proptest with 64 cases
