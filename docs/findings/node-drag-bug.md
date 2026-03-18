# Node Drag Bug Findings (2026-03-18)

## Issue
When interacting with the canvas, grabbing and dragging a node would fail to drop it at the new location. The node would either instantly snap back to its origin or fail to update its position, even though Playwright traces showed that mouse coordinates were being generated correctly.

## Root Causes

Two separate state-management bugs overlapped to break the pointer drop interactions:

1. **Local State Shadowing in `NodeLayer`**
   In `diagram_tool/src/ui/canvas/node_layer/mod.rs`, the layer had this line:
   ```rust
   let pending_pointer_sample = use_signal(|| Option::<(f64, f64)>::None);
   ```
   Instead of using the globally tracked pointer position from the `CanvasState`, the component tree was creating a completely fake local signal and threading it into the `NodeElement`. As a result, when the mouse button was released, `flush_pending_pointer_update` was called with this empty local signal. It saw `None` for the cursor position and aborted the position update entirely.

2. **Premature State Cancellation in Pointer Bridge**
   In `diagram_tool/src/ui/canvas/root_handlers/pointer/up.rs`, the global `pointerup` bridge event handler (designed to catch out-of-bounds mouse releases) contained this logic:
   ```rust
   if was_captured {
       deps.interaction_mode.set(InteractionMode::Select);
   }
   
   flush_pending_pointer_update(...);
   ```
   This forced the interaction mode back to `Select` *before* `flush_pending_pointer_update` had a chance to run. When the system attempted to flush the final drag, it saw that the system was in `Select` mode and assumed there was no active dragging to complete!

## Fixes Applied

1. **Threaded the true state to NodeLayer**
   - Removed the shadowed `use_signal` in `NodeLayer`
   - Added `pending_pointer_sample: Signal<Option<(f64, f64)>>` as a prop in `NodeLayer`'s signature
   - Passed `state.pending_pointer_sample` down correctly from `RootContainer`.

2. **Fixed Pointer Bridge Ordering**
   - Removed the premature `deps.interaction_mode.set(InteractionMode::Select)` call before the pointer flush.
   - Refactored `handle_pointer_up` so that `flush_pending_pointer_update` evaluates the real dragging interaction mode.
   - Allowed the `match` statement inside `handle_pointer_up` to cleanly reset the mode to `Select` *after* the drag finishes.

3. **Playwright E2E Enhancements**
   - Discovered that using pure `page.mouse.move()` without `pointerId` payloads would trigger some issues with the virtual pointer bridge.
   - Updated the E2E drag test to use `node.dragTo(canvasArea)` which is natively supported by Playwright and simulates proper browser pointer events more closely. 
   - Strengthened test stability by using Playwright's `domcontentloaded` instead of `load` in the initialization phase.

## Validation
- Ran `moon run :serve-e2e` and confirmed manual dragging works successfully.
- Re-ran Playwright test `node_drag.debug.spec.ts` directly, passing with correct coordinate threshold validation.
