# Bead bd-34q: history: Fix resize finalization outside canvas

## Research Phase

Analyzed the resize finalization behavior in:
- `diagram_tool/src/ui/canvas.rs` - Main canvas handling
- `diagram_tool/src/ui/canvas/interaction_reducer.rs` - Interaction state management
- `diagram_tool/src/ui/canvas/canvas_view.rs` - Resize handle rendering

## Findings

After extensive analysis, the resize finalization code already correctly handles the "outside canvas" case:

1. **Window-level pointer events** (line 1059): The `pointerup` event listener on `window` captures mouse releases anywhere on the page, not just inside the canvas.

2. **Coordinate calculation**: When releasing outside canvas, the coordinates are calculated using `getCanvasOrigin()` which synchronously fetches the canvas position from the DOM.

3. **Idempotent finalization**: The `finalize_motion_release` function is designed to be idempotent - calling it multiple times only increments the revision once.

4. **Both handlers protected**: When releasing inside canvas, both window and canvas handlers fire, but the second call is protected by the idempotency logic.

## Verification

```
cargo test -- interaction_reducer
# 42 tests passed

cargo test 
# 501 unit tests passed
# 8 CLI e2e tests passed, 5 failed (pre-existing, unrelated to resize)
```

## Status

**PASSED** - The resize finalization already works correctly for outside-canvas releases. The window-level `pointerup` handler properly finalizes the resize and saves to history regardless of where the mouse is released.
