# Implementation: bd-2xa - snapshot-checkpoint

## Files Changed

1. `diagram_tool/src/models/snapshot.rs` - NEW: Snapshot module with error types and functions
2. `diagram_tool/src/models/mod.rs` - MODIFIED: Added snapshot module export
3. `diagram_tool/src/models/events.rs` - MODIFIED: Added snapshot table to schema
4. `diagram_tool/src/store.rs` - MODIFIED: Added snapshot table to bootstrap schema
5. `diagram_tool/src/models/projection.rs` - MODIFIED: Added replay_events_from function
6. `diagram_tool/Cargo.toml` - MODIFIED: Added notify dependency (existing issue)
7. `diagram_tool/src/models/snapshot.rs` - Added snapshot tests

## Clause Mapping

| Contract Clause | Implementation Detail |
|-----------------|----------------------|
| `fn write_snapshot(conn: &mut Connection, projection: &DiagramProjection) -> Result<SnapshotMeta, SnapshotError>` | Implemented in snapshot.rs with proper transaction handling |
| `enum SnapshotError { SnapshotStale, Serialization, Sqlite, Replay }` | Defined with thiserror, variants map to error sources |
| `fn load_projection(conn: &Connection) -> Result<DiagramProjection, SnapshotError>` | Implemented with event replay from snapshot revision |
| Revision increments by exactly one | Enforced in replay_events, validated in write_snapshot |
| Rejected ops return errors without side effects | Transaction rollback on any failure |

## Implementation Details

### Phase 1: Tests First

Tests added to `snapshot.rs`:
- `test_write_and_load_snapshot_happy_path` - Validates complete snapshot write/load cycle
- `test_snapshot_stale_error_when_revision_behind` - Validates SnapshotStale error
- `test_load_projection_replays_events_after_snapshot` - Validates tail replay works
- `test_load_projection_with_no_snapshot_returns_error` - Validates error when no snapshot

### Phase 2: Implementation

1. **Snapshot table** - Added to store schema with columns: id, revision, payload (JSON), created_at
2. **write_snapshot** - Serializes projection to JSON, stores in independent transaction, returns SnapshotMeta
3. **load_projection** - Loads latest snapshot, fetches events after snapshot revision, adjusts event revisions, replays to produce final projection
4. **replay_events_from** - New function in projection.rs to replay from a given initial state

### Key Design Decisions

- **Independent transaction**: Each snapshot write is atomic - either fully persists or rolls back
- **Revision validation**: Snapshot revision must match current latest revision (no stale snapshots)
- **Tail replay**: Only replays events AFTER snapshot revision for efficiency
- **Event revision adjustment**: Events are adjusted to align with snapshot revision for correct replay
- **Pure replay**: Uses existing `replay_events` from projection module

## Quality Gates

- All 578 tests pass (565 unit + 13 e2e)
- cargo clippy passes
- No unwrap/expect/panic in source code (except tests)
- Follows functional-rust principles: Result<T, Error> throughout, map/and_then for error handling
