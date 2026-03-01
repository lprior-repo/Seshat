bead_id: bd-x3x
bead_title: sync-ui-apply: batch apply tail events without render blocking
phase: p0
updated_at: 2026-03-01T20:28:40Z

# Contract: sync-ui-apply

## Summary

Batch apply tail events from the sync watcher to the UI projection without
blocking the render loop. This enables the GUI to stay responsive while
processing external CLI writes detected by the file watcher.

## Preconditions

### System State
- SQLite connection is open with WAL enabled and synchronous FULL
- `start_event_tail_watcher` is active and sending `SyncMessage` notifications
- `fetch_new_events` can retrieve events after a given revision

### Rust Contract Signature
```rust
fn apply_tail_batch(
    state: &mut ProjectionState,
    events: Vec<EventRecord>
) -> Result<ApplySummary, SyncError>;
```

### Rust Error Contract
```rust
enum SyncError {
    Replay,
    ChannelClosed,
    Backpressure,
}
```

## Postconditions

### State Changes
- `ProjectionState` is updated with all new events applied
- UI receives minimal signal updates via `schedule_ui_update`
- Render loop is not blocked during batch processing

### Rust Postcondition Signature
```rust
fn schedule_ui_update(summary: ApplySummary) -> Result<(), SyncError>;
```

## Invariants

1. No migration path is introduced
2. No dual-write compatibility path exists
3. All fallible operations use typed Result errors
4. Batch processing happens off the render hot path
5. UI updates are scheduled, not immediate

## Implementation Tasks

1. Batch replay updates off render hot path
2. Publish minimal signal updates to Dioxus state

## Acceptance Criteria

- All sync module tests pass
- Batch apply processes events without blocking UI
- Signal updates are minimal and targeted
- Error handling uses typed Result throughout

## Related Files

- `diagram_tool/src/models/sync.rs` - Sync watcher and fetch functions
- `diagram_tool/src/models/projection.rs` - Projection state and replay
- `diagram_tool/src/ui/` - UI components and state management
