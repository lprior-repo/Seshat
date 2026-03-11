# Martin Fowler Test Plan: seshat-7fz

## Happy Path Tests

### test_detects_dropped_ai_event_when_not_in_wal
- **Given**: `pending_ai_ops` contains AI op_id "ai-op-1", WAL has no events after last revision
- **When**: `detect_dropped_ai_events()` is called
- **Then**:
  - Returns `Ok(["ai-op-1"])`
  - `ai_conflict_state` is updated to `Some("AI operation rejected - human has active edit")`
  - `pending_ai_ops` no longer contains "ai-op-1"

### test_no_conflict_when_ai_event_appears_in_wal
- **Given**: `pending_ai_ops` contains AI op_id "ai-op-1", WAL contains the event with op_id "ai-op-1"
- **When**: `detect_dropped_ai_events()` is called
- **Then**:
  - Returns `Ok([])` (empty - no drops detected)
  - `ai_conflict_state` remains unchanged
  - "ai-op-1" remains in `pending_ai_ops`

### test_human_events_ignored_in_pending_tracking
- **Given**: A human event is dispatched (id starts with "human-")
- **When**: Event appears in pending set
- **Then**:
  - Human events are NOT tracked in `pending_ai_ops` (only AI events)

---

## Error Path Tests

### test_returns_error_when_store_bridge_unavailable
- **Given**: StoreBridge pool is None (not initialized)
- **When**: `detect_dropped_ai_events()` is called
- **Then**: Returns `Err(Error::StoreUnavailable("Pool not initialized"))`

### test_returns_error_when_fetch_fails
- **Given**: StoreBridge is valid but `fetch_events_since_sync` returns error
- **When**: `detect_dropped_ai_events()` is called
- **Then**: Returns `Err(Error::FetchFailed("..."))`

### test_handles_empty_pending_set_gracefully
- **Given**: `pending_ai_ops` is empty
- **When**: `detect_dropped_ai_events()` is called
- **Then**: Returns `Ok([])`, no state changes

---

## Edge Case Tests

### test_multiple_dropped_ai_events_detected_together
- **Given**: `pending_ai_ops` contains ["ai-op-1", "ai-op-2", "ai-op-3"], none appear in WAL
- **When**: `detect_dropped_ai_events()` is called
- **Then**:
  - Returns all 3 dropped op_ids
  - `conflict_state` updated with message
  - All 3 removed from pending

### test_mixed_dropped_and_confirmed_events
- **Given**: `pending_ai_ops` contains ["ai-op-1", "ai-op-2"], WAL contains "ai-op-1" only
- **When**: `detect_dropped_ai_events()` is called
- **Then**:
  - Returns only ["ai-op-2"]
  - "ai-op-1" remains in pending, "ai-op-2" removed

### test_poll_with_revision_gap_detects_missing
- **Given**: Last revision was 5, new events have revision 7 (gap at 6)
- **When**: Poller detects gap
- **Then**: Error returned or gap handling per existing revision gap policy

---

## Contract Verification Tests

### test_precondition_p1_ai_conflict_state_exists
- **Given**: Dioxus context has `ai_conflict_state` signal
- **When**: Poller runs
- **Then**: Can access signal without panic

### test_precondition_p2_store_bridge_valid
- **Given**: StoreBridge is initialized with valid pool
- **When**: Poller runs fetch
- **Then**: No StoreUnavailable error

### test_postcondition_q1_conflict_state_updated_on_drop
- **Given**: AI event was dropped (not in WAL)
- **When**: Poller detects drop
- **Then**: `ai_conflict_state.read()` returns `Some(_)`

### test_postcondition_q2_pending_cleared_after_detection
- **Given**: AI op_id in pending set
- **When**: Drop detected for that op_id
- **Then**: op_id no longer in pending set

### test_invariant_i1_only_ai_ops_in_pending
- **Given**: Various operations added to pending
- **When**: After any mutation
- **Then**: All entries are AI-authored (verify via is_human_author check)

### test_invariant_i3_atomic_poller_operation
- **Given**: Poller running
- **When**: Multiple concurrent poll attempts
- **Then**: No race conditions (ensure proper signal locking)

---

## Contract Violation Tests

### test_violation_p1_panic_without_signal_context
- **Given**: App component without `ai_conflict_state` in context
- **When**: Poller attempts to access signal
- **Then**: Panics with "ai_conflict_state signal not found in context"

### test_violation_p2_store_unavailable_returns_error
- **Given**: StoreBridge pool is None
- **When**: `fetch_events_since_sync` called
- **Then**: Returns `Err(Error::StoreUnavailable(_))`

### test_violation_q1_conflict_not_updated_when_dropped
- **Given**: AI event in pending, not in WAL
- **When**: Poller runs but fails to update conflict_state
- **Then**: Test should fail - this is the bug we're fixing

### test_violation_q2_pending_not_cleared
- **Given**: AI event detected as dropped
- **When**: After detection
- **Then**: op_id must be removed from pending (verify via contains check)

---

## Given-When-Then Scenarios

### Scenario 1: AI Edit Rejected Due to Human Priority
**Given**:
- User (human) is editing node "node-a" in the UI
- AI dispatches an operation to move "node-a" with op_id "ai-move-1"
- Operation is appended but rejected due to revision mismatch (human changed it)
- "ai-move-1" is in `pending_ai_ops`

**When**:
- Poller runs (fetches events since last revision)
- No event with op_id "ai-move-1" appears in fetched events

**Then**:
- `detect_dropped_ai_events` returns `["ai-move-1"]`
- `ai_conflict_state` is set to `Some("AI operation rejected - human has active edit")`
- "ai-move-1" is removed from `pending_ai_ops`

### Scenario 2: AI Edit Successfully Applied
**Given**:
- AI dispatches node creation with op_id "ai-create-1"
- "ai-create-1" is added to `pending_ai_ops`
- No human edits conflict

**When**:
- Poller runs, fetches events including "ai-create-1"

**Then**:
- Returns empty list (no drops)
- `ai_conflict_state` unchanged
- "ai-create-1" remains in pending until confirmed (or could be removed on appearance)

### Scenario 3: Multiple Rapid AI Operations
**Given**:
- AI dispatches 5 operations rapidly: ["ai-op-1", "ai-op-2", "ai-op-3", "ai-op-4", "ai-op-5"]
- All added to `pending_ai_ops`
- Human edits entity affected by "ai-op-3"

**When**:
- Poller fetches events - "ai-op-3" was rejected, others succeeded

**Then**:
- Returns `["ai-op-3"]`
- Conflict message shown once
- Only "ai-op-3" removed from pending

---

## Implementation Phases

### Phase 1: Track Pending AI Operations
- Add `pending_ai_ops: Signal<HashSet<String>>` to app context
- When AI event dispatched, add op_id to pending (filter: only AI)
- Human events bypass pending tracking

### Phase 2: Implement Drop Detection
- Create `detect_dropped_ai_events()` function
- Fetch events since last revision
- Compare fetched op_ids against pending set
- Identify missing (dropped) ops

### Phase 3: Signal Integration
- Update `ai_conflict_state` when drops detected
- Clear detected ops from pending
- Ensure proper error handling

### Phase 4: Integration with Existing Poller
- Wire into existing 500ms poller in app.rs
- Use existing store_bridge and revision tracking
- Ensure no regression in existing event application
