# Contract Specification: seshat-7fz

## Context

- **Feature**: UI Conflict: Poller Detection
- **Bead ID**: seshat-7fz
- **Description**: Update the background WAL poller to detect dropped EventEnvelopes and update the ai_conflict_state signal.
- **Domain Terms**:
  - `EventEnvelope`: Structure containing `op_id`, `author`, `operation`, `timestamp`
  - `Author`: Contains `id` and `name`; human authors have `id` starting with "human-" or name containing "human"
  - `ai_conflict_state`: Dioxus Signal<Option<String>> tracking conflict messages
  - `StoreBridge`: Sync bridge to async WAL store operations
  - `fetch_events_since_sync(revision)`: Fetches events from WAL since given revision
  - `pending_ai_ops`: Set of AI operation op_ids that have been dispatched but not confirmed in WAL
- **Assumptions**:
  - AI events are identified by `!is_human_author(&author)` where human = id.starts_with("human-") or name contains "human"
  - Dropped events = AI events that were dispatched but never appeared in WAL after a poll cycle
  - The poller runs every 500ms in the UI coroutine
  - `ai_conflict_state` signal already exists (from seshat-6jd)
- **Open Questions**:
  - Should we track all AI events or only those with expected_revision set?
  - How long should we wait before considering an event "dropped"?
  - Should we deduplicate conflict messages if multiple AI ops are dropped?

---

## Preconditions

- **P1**: `ai_conflict_state` signal is available in Dioxus context
  - Enforcement: Runtime - panic if not found in context (UI invariant)
- **P2**: `StoreBridge` is initialized and provides `fetch_events_since_sync`
  - Enforcement: Runtime - check `store_bridge` is Some and valid
- **P3**: Poller has valid `last_sync_revision` state
  - Enforcement: Runtime - initial value must be >= 0

---

## Postconditions

- **Q1**: When an AI event is detected as dropped, `ai_conflict_state` is updated to `Some(message)`
  - Message format: "AI operation rejected - human has active edit" or similar descriptive text
- **Q2**: Dropped AI operations are removed from `pending_ai_ops` after detection
  - Prevents repeated conflict notifications for same operation
- **Q3**: When no conflicts detected, `ai_conflict_state` remains unchanged (not cleared by poller)
  - Clearing is handled by separate auto-dismiss bead (seshat-fsa)

---

## Invariants

- **I1**: `pending_ai_ops` only contains AI-authored operations (never human)
- **I2**: Every entry in `pending_ai_ops` has a unique `op_id`
- **I3**: Poller runs atomically - no race between fetch and state update
- **I4**: Signal updates are monotonic - newer conflict message replaces older

---

## Error Taxonomy

- **Error::StoreUnavailable**: StoreBridge not initialized or pool not acquired
- **Error::FetchFailed**: `fetch_events_since_sync` returned error (network, DB issues)
- **Error::ParseFailed**: Failed to parse event envelope from payload
- **Error::PreconditionViolated**: Required context signals not available

---

## Contract Signatures

```rust
/// Detects dropped AI events and updates conflict state
/// 
/// # Parameters
/// - `store_bridge`: Reference to StoreBridge for fetching events
/// - `last_revision`: Current sync revision tracker (Signal<i64>)
/// - `pending_ops`: Set of pending AI operation op_ids (Signal<HashSet<String>>)
/// - `conflict_state`: Signal<Option<String>> for conflict messages
/// 
/// # Returns
/// - `Result<Vec<String>, Error>` - List of dropped op_ids that were detected
/// 
/// # Errors
/// - Returns `Err(Error::StoreUnavailable)` if store bridge is invalid
/// - Returns `Err(Error::FetchFailed)` if WAL fetch fails
fn detect_dropped_ai_events(
    store_bridge: &StoreBridge,
    last_revision: &Signal<i64>,
    pending_ops: &Signal<HashSet<String>>,
    conflict_state: &Signal<Option<String>>,
) -> Result<Vec<String>, Error>;
```

---

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: ai_conflict_state exists | Runtime | `use_context::<Signal<Option<String>>>()` - panic if None |
| P2: StoreBridge valid | Runtime | `store_bridge.is_some()` check before use |
| P3: last_revision >= 0 | Compile-time | `NonNegativeI64` wrapper or debug_assert |

---

## Violation Examples (REQUIRED)

- **VIOLATES P1**: Calling poller detection without `ai_conflict_state` in context
  - Should panic with "ai_conflict_state signal not found in context"
  
- **VIOLATES P2**: StoreBridge not initialized (pool is None)
  - Should return `Err(Error::StoreUnavailable("Pool not initialized"))`

- **VIOLATES Q1**: AI event dropped but conflict_state not updated
  - Test: Send AI op, verify it's in pending, poll with no events, verify conflict_state has Some value

- **VIOLATES Q2**: Dropped op_id not removed from pending set
  - Test: After conflict detected, pending_ops should not contain the dropped op_id

---

## Ownership Contracts (Rust-specific)

- **StoreBridge**: Shared reference `&Arc<StoreBridge>` - no mutation, read-only access to WAL
- **Signal<i64> (last_revision)**: Exclusive borrow via `.write()` - mutation: value updated to max fetched revision
- **Signal<HashSet<String>> (pending_ops)**: Exclusive borrow via `.write()` - mutation: entries added on dispatch, removed on detection
- **Signal<Option<String>> (conflict_state)**: Exclusive borrow via `.write()` - mutation: set to Some on conflict detection

---

## Non-goals

- [ ] Persisting conflict state across app restarts
- [ ] Multiple concurrent pollers (single poller is invariant)
- [ ] Conflict resolution UI (handled by seshat-3t5)
- [ ] Auto-dismiss of conflict messages (handled by seshat-fsa)
- [ ] Tracking revision gaps unrelated to AI conflicts
