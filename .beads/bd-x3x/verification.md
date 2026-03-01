bead_id: bd-x3x
bead_title: sync-ui-apply: batch apply tail events without render blocking
phase: p2
updated_at: 2026-03-01T20:46:00Z

# Verification: sync-ui-apply

## Moon Validation Results

### Test Execution

```
cargo test -p diagram_tool models::sync::

running 21 tests
test models::sync::tests::test_apply_tail_batch_with_empty_events_returns_empty_summary ... ok
test models::sync::tests::test_schedule_ui_update_with_empty_summary_succeeds ... ok
test models::sync::tests::test_start_event_tail_watcher_fails_for_nonexistent_path ... ok
test models::sync::tests::test_start_store_watcher_fails_for_nonexistent_path ... ok
test models::sync::tests::test_schedule_ui_update_with_events_succeeds ... ok
test models::sync::tests::test_fetch_latest_revision_returns_zero_when_empty ... ok
test models::sync::tests::test_fetch_new_events_returns_empty_when_no_events ... ok
test models::sync::tests::test_event_record_contains_correct_data ... ok
test models::sync::tests::test_fetch_new_events_returns_all_events_when_after_revision_zero ... ok
test models::sync::tests::test_stop_store_watcher_succeeds ... ok
test models::sync::tests::test_watcher_handle_is_active_flag ... ok
test models::sync::tests::test_replaying_fetched_events_produces_correct_projection ... ok
test models::sync::tests::test_fetch_new_events_returns_events_after_revision ... ok
test models::sync::tests::test_start_store_watcher_succeeds_for_existing_db ... ok
test models::sync::tests::test_apply_tail_batch_extracts_affected_entities ... ok
test models::sync::tests::test_events_are_ordered_by_revision ... ok
test models::sync::tests::test_fetch_latest_revision_returns_max_revision ... ok
test models::sync::tests::test_apply_tail_batch_applies_events_and_updates_revision ... ok
test models::sync::tests::test_fetch_new_events_returns_empty_when_after_revision_is_latest ... ok
test models::sync::tests::test_start_event_tail_watcher_succeeds_for_existing_db ... ok
test models::sync::tests::test_watcher_detects_database_modifications ... ok

test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured
```

## Contract Verification

### Preconditions Met

- [x] SQLite connection is open with WAL enabled and synchronous FULL
- [x] `start_event_tail_watcher` is active and can send notifications
- [x] `fetch_new_events` can retrieve events after a given revision

### Postconditions Verified

- [x] `DiagramProjection` is updated with all new events applied
- [x] `ApplySummary` contains correct revision information
- [x] Affected entities are correctly extracted
- [x] UI update scheduling function returns success

### Invariants Preserved

- [x] No migration path introduced
- [x] No dual-write compatibility path exists
- [x] All fallible operations use typed Result errors (`SyncError::Decode`)
- [x] Batch processing happens off render hot path
- [x] UI updates are scheduled, not immediate

## New Function Tests

### apply_tail_batch

| Test | Description | Result |
|------|-------------|--------|
| `test_apply_tail_batch_with_empty_events_returns_empty_summary` | Empty events produce empty summary | PASS |
| `test_apply_tail_batch_applies_events_and_updates_revision` | Events applied, revision updated | PASS |
| `test_apply_tail_batch_extracts_affected_entities` | Entity IDs extracted correctly | PASS |

### schedule_ui_update

| Test | Description | Result |
|------|-------------|--------|
| `test_schedule_ui_update_with_empty_summary_succeeds` | Empty summary handled | PASS |
| `test_schedule_ui_update_with_events_succeeds` | Non-empty summary handled | PASS |

## Code Quality

- No clippy errors in new code
- All functions documented with rustdoc
- Error handling uses typed Result throughout
- No unwrap or expect in production code paths

## Summary

All verification checks pass. The implementation satisfies the contract
requirements for `bd-x3x`.
