# Implementation Summary: bd-1yu - history: guarantee gesture atomicity

## Contract Reference
- **ears_requirements**: 
  - ubiquitous: One completed user gesture maps to one history entry
  - event_driven: Duplicate pointerup/mouseup events finalize at most once
  - unwanted: Blur splitting nudge sequence should NOT merge gestures

## Implementation Status: COMPLETE

### Phase 1: Tests Added ✅
- Added 5 new regression tests in `interaction_reducer.rs` to verify gesture atomicity

### Phase 2: Implementation Verified ✅
- The existing `finalize_motion_release` function already correctly handles idempotency:
  - Returns `false` on subsequent calls after first successful finalize
  - Only increments revision once per gesture
  - Transitions mode to `Select` after first call, making subsequent calls no-ops

## Files Changed

### diagram_tool/src/ui/canvas/interaction_reducer.rs
Added tests:
1. `given_already_in_select_mode_when_finalized_then_no_revision_change` - Tests idempotent behavior when duplicate events arrive after gesture already completed
2. `given_drag_gesture_when_duplicate_events_arrive_then_history_single_entry` - Simulates E2E scenario with normal mouseup + duplicate pointerup/mouseup
3. `given_resize_gesture_when_duplicate_events_arrive_then_history_single_entry` - Similar test for resize gestures
4. `given_no_op_gesture_when_finalized_then_no_revision_bump` - Verifies no revision bump for no-op gestures
5. `given_mixed_gesture_sequence_when_finalized_then_correct_revisions` - Tests realistic sequence of multiple gestures

## Test Results
```
test result: ok. 494 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

All 494 tests pass (489 original + 5 new = 494).

## Contract Satisfaction

| Requirement | Status | Evidence |
|-------------|--------|----------|
| One gesture = one history entry | ✅ | `finalize_motion_release` idempotent; tests verify single revision bump |
| Duplicate events finalize at most once | ✅ | Tests verify revision unchanged after first finalize |
| Blur splits nudge sequences | ✅ | Implemented via `nudge_batch_active` signal in canvas.rs (lines 581, 668, 697, 746, 769) |

## Notes
- The implementation already handles the race conditions correctly in the reducer
- The `did_move` and `did_resize` flags in the interaction modes prevent duplicate history pushes
- The `finalize_motion_release` function transitions to `Select` mode on first call, making subsequent calls no-ops
- Nudge batch handling is correctly implemented: blur resets `nudge_batch_active`, causing each nudge after blur to create a new undo entry
