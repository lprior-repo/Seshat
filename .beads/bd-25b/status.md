# Bead bd-25b: selection - Fix marquee not clearing existing selection

## Status: passed

## Evidence

### Code Change
- File: `/home/lewis/src/seshat/diagram_tool/src/ui/canvas.rs`
- Added explicit clearing of selection before applying marquee selection when additive mode is false

### Test Results
```
$ cargo test --bin diagram_tool
running 502 tests
test ui::canvas::tests::given_existing_selection_when_rubber_band_released_then_selection_is_cleared ... ok
test ui::canvas::tests::given_noop_rubber_band_when_released_then_selection_is_preserved ... ok
test ui::canvas::tests::given_rubber_band_release_when_applied_then_selection_is_committed ... ok

test result: ok. 502 passed; 0 failed; 0 ignored; 0 measured
```

### Fix Explanation
The bug was that when using marquee (rubber band) selection without holding Shift/Ctrl/Meta (additive mode), the existing selection was not being cleared before applying the new selection.

The fix adds an explicit `doc.editor_state.selected_items.clear()` call in the `apply_rubber_band_release` function when `additive` is false, ensuring the existing selection is cleared before the new marquee selection is applied.

This ensures:
1. When user drags a marquee without additive key, existing selection is cleared first
2. New selection contains only nodes within the marquee
3. When dragging with additive key (Shift/Ctrl/Meta), existing selection is preserved and nodes in marquee are toggled
4. Clicking without dragging (noop rubber band) still preserves selection as expected
