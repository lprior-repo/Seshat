# Bead bd-345: Tests: Fix failing embedding scroll tests

## Status: FAILED

### Summary
The embedding scroll offset tests continue to fail despite implementing a fix for the race condition. The fix was conceptually correct but I was unable to verify it due to test infrastructure issues (the `dx` CLI command is not available in this environment to start the test server).

### Root Cause
The race condition is:
- The Dioxus `onmousedown` handler reads from the `canvas_origin` signal
- This signal is updated asynchronously via messages from JavaScript (pollOrigin runs in requestAnimationFrame and sends updates via dioxus.send())
- When the user clicks after scrolling, the onmousedown can fire before the signal is updated with the new position
- The pointermove/pointerup handlers work correctly because they include the origin directly in the JavaScript message

### Changes Made (Conceptually Correct Fix)
1. **JavaScript pointerdown handler:**
   - Added `event.preventDefault()` and `event.stopPropagation()` to prevent the Dioxus onmousedown from running
   - Send a 'pointerdown' message with the fresh origin directly computed from the DOM

2. **Rust message handler:**
   - Added handling for 'pointerdown' event type that uses the origin from the message directly
   - Added 'resize' message handling in the second spawn to handle resize messages from pointerdown
   - The message now carries originX and originY computed synchronously in JavaScript

3. **The fix works as follows:**
   - User clicks canvas
   - JavaScript pointerdown fires, computes fresh origin from DOM, sends message with origin
   - Rust message handler receives message with fresh origin, creates text node at correct position
   - Dioxus onmousedown is prevented from running (no stale coordinate issue)

### Test Results (Before Infrastructure Issues)
- The fix was not verified due to `dx` CLI not being available to start the test server
- Code compiles successfully
- The test error showed X coordinate working but Y being off by 680 pixels, suggesting the origin wasn't updating after scroll

### Evidence of Changes
The git diff shows:
- Modified JavaScript pointerdown to prevent Dioxus handler and send complete event data
- Added Rust handling for pointerdown that uses origin from message directly
- Added resize handling in the second spawn for the pointerdown-triggered updates
- Code compiles without errors

### Next Steps
The fix should work in principle. To verify:
1. Ensure the test server can start (`dx serve` or `moon run :serve-e2e`)
2. Run the embedding scroll tests
3. If still failing, check that the pointerdown handler is actually being triggered
4. Consider adding console.log debugging to verify the message flow
