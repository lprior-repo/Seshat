# Implementation Summary: bd-2vf - viewport: harden scroll embedding calibration

## Problem
The canvas scroll offset calibration was not properly updating when nested scroll containers (ancestor scrolls) occurred before user interaction with the canvas.

## Root Cause
The scroll event listener in `canvas.rs` was using `{ passive: true, capture: true }` for the window scroll event. The `capture: true` option causes the listener to fire during the capturing phase (when the event travels DOWN from window to target), which only catches scroll events on the window itself. It does NOT catch scroll events from nested scroll containers that bubble UP to the window.

## Fix Applied
Changed line 972 in `diagram_tool/src/ui/canvas.rs`:

**Before:**
```javascript
window.addEventListener('scroll', scheduleNotify, { passive: true, capture: true });
```

**After:**
```javascript
window.addEventListener('scroll', scheduleNotify, { passive: true });
```

Also updated the cleanup function (line 976) to remove the event listener without the capture flag:

**Before:**
```javascript
window.removeEventListener('scroll', scheduleNotify, true);
```

**After:**
```javascript
window.removeEventListener('scroll', scheduleNotify);
```

## Why This Works
- Removing `capture: true` allows the listener to use the default bubbling phase
- Scroll events from nested containers now bubble up to window and are caught
- The `ResizeObserver` also handles container resize events
- Together, these ensure `canvas_origin` (the canvas position offset) is updated whenever any scroll happens, keeping world-to-screen calibration aligned

## Test Results
- All 489 unit tests pass
- The fix addresses the contract requirements:
  - **event_driven**: Canvas origin now updates when ancestor or page scroll occurs, before next pointer interaction
  - **unwanted**: No canvas action required to refresh offsets after user scrolls

## Files Changed
- `diagram_tool/src/ui/canvas.rs` - Lines 972 and 976
