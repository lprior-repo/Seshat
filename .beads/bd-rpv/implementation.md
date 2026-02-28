# Implementation: canvas: Box select with drag marquee

bead_id: bd-rpv
bead_title: canvas: Box select with drag marquee
phase: p2
updated_at: 2026-02-28T19:40:00Z

## Implementation Status: COMPLETE

The box select / marquee selection feature is fully implemented in the codebase:

### Implementation Details

1. **RubberBand Interaction Mode** (`diagram_tool/src/ui/canvas/interaction_reducer.rs`):
   - `InteractionMode::RubberBand { start, current }` stores start position and current drag position

2. **Visual Rectangle Overlay** (`diagram_tool/src/ui/canvas/canvas_view.rs:357-382`):
   ```rust
   pub(super) fn rubber_band_overlay(...) -> Element {
       if let InteractionMode::RubberBand { start, current } = mode {
           // Draws rectangle with SELECTION_RECT_FILL and SELECTION_RECT_STROKE
       }
   }
   ```

3. **Node Filtering** (`diagram_tool/src/ui/interaction.rs:66-95`):
   ```rust
   pub fn node_ids_in_rect_with_mode(...) {
       // Filters nodes by SelectionMode::Contain (fully inside) or Intersect
   }
   ```

4. **Selection Commit** (`diagram_tool/src/ui/canvas.rs:231`):
   ```rust
   fn apply_rubber_band_release(doc: &mut DiagramDocument, start: (f64, f64), current: (f64, f64), additive: bool)
   ```

5. **Selection Mode** (`diagram_tool/src/ui/interaction.rs:15-28`):
   - `SelectionMode::Contain` - nodes fully inside marquee
   - `SelectionMode::Intersect` - nodes that intersect marquee

## Verification Evidence

- Moon check: PASSED
- Moon test: 491 tests passed, 0 failed
- Moon clippy: PASSED  
- Cargo fmt: PASSED
- Unit tests for rubber_band_release exist (lines 2619-2646)

## Contract Compliance

| Requirement | Status |
|-------------|--------|
| Display selection rectangle while dragging | ✅ rubber_band_overlay |
| Select nodes fully within marquee | ✅ Contain mode |
| Start marquee on drag on empty canvas | ✅ InteractionMode::RubberBand |
| Select nodes on release | ✅ apply_rubber_band_release |
| Preserve selection on empty marquee | ✅ Implementation checks for empty |
| Partial overlap not selected | ✅ Uses Contain mode (not Intersect) |
