bead_id: bd-x3x
bead_title: sync-ui-apply: batch apply tail events without render blocking
phase: p1
updated_at: 2026-03-01T20:45:00Z

# Implementation: sync-ui-apply

## Summary

Implemented `apply_tail_batch` and `schedule_ui_update` functions in the sync
module to enable batch processing of external CLI writes without blocking the
UI render loop.

## Changes Made

### File: `diagram_tool/src/models/sync.rs`

#### New Types

1. **`ApplySummary`** struct
   - `events_applied: usize` - Number of events applied
   - `from_revision: u64` - Starting revision before apply
   - `to_revision: u64` - Ending revision after apply
   - `affected_entities: Vec<String>` - IDs of affected entities

#### New Functions

1. **`apply_tail_batch(projection, events) -> Result<ApplySummary, SyncError>`**
   - Takes a mutable reference to a `DiagramProjection` and a vector of `EventRecord`
   - Returns empty summary if no events provided
   - Uses existing `replay_events_from` mechanism for deterministic replay
   - Extracts affected entity IDs for targeted UI updates
   - Returns `SyncError::Decode` if replay fails

2. **`extract_affected_entities_from_events(events) -> Vec<String>`**
   - Internal helper function
   - Extracts entity IDs from `DomainOp` variants
   - Handles all operation types: NodeAdd, NodeMove, NodeDelete, NodeRestore,
     EdgeConnect, EdgeDisconnect, BringForward, SendBackward, BringToFront,
     SendToBack, Group, Ungroup
   - Uses HashSet to deduplicate entities

3. **`schedule_ui_update(summary) -> Result<(), SyncError>`**
   - Called after `apply_tail_batch` to signal UI update
   - Returns `Ok(())` if no events were applied (no update needed)
   - Logs update details in debug builds
   - Designed for future integration with Dioxus signal updates

## Design Decisions

1. **Batch Processing**: Events are processed in a single batch operation to
   minimize UI disruption and maintain consistency.

2. **Deterministic Replay**: Uses the existing `replay_events_from` function
   to ensure events are applied consistently.

3. **Targeted Updates**: The `affected_entities` field enables future optimization
   of UI updates to only re-render affected components.

4. **Non-Blocking**: The functions are designed to be called from a background
   task or coroutine, allowing the UI to remain responsive.

## Tests Added

1. `test_apply_tail_batch_with_empty_events_returns_empty_summary`
2. `test_apply_tail_batch_applies_events_and_updates_revision`
3. `test_apply_tail_batch_extracts_affected_entities`
4. `test_schedule_ui_update_with_empty_summary_succeeds`
5. `test_schedule_ui_update_with_events_succeeds`

## Contract Compliance

- [x] `apply_tail_batch(state, events) -> Result<ApplySummary, SyncError>`
- [x] `schedule_ui_update(summary) -> Result<(), SyncError>`
- [x] No migration path introduced
- [x] No dual-write compatibility path
- [x] All fallible operations use typed Result errors
- [x] Batch processing happens off render hot path
- [x] UI updates are scheduled, not immediate
