bead_id: bd-2md
bead_title: edge-case-bdd-tests-concurrent-access
phase: p0
updated_at: 2026-03-02T05:24:00Z

# Contract: BDD Tests for Concurrent Access Edge Cases

## Scope

Add comprehensive BDD-style tests for concurrent access scenarios in `diagram_tool/src/locking/`. Tests must cover edge cases related to lock timeouts, concurrent operations on the same diagram, lock release failures, and queue overflow conditions that could cause deadlocks, data corruption, or resource exhaustion.

## Test Categories

### 1. Lock Timeout Scenarios

| Test Case | Given | When | Then |
|-----------|-------|------|------|
| `given_held_lock_when_timeout_expires_then_returns_timeout_error` | Lock is held by another operation | Second lock attempt with short timeout | Returns `LockError::Timeout` with descriptive message |
| `given_expired_lock_when_second_attempt_then_succeeds` | First lock holder releases | Second attempt acquires lock | Successfully acquires lock |
| `given_zero_timeout_when_lock_contended_then_fails_immediately` | Lock is held | Acquisition with zero timeout | Fails immediately without retry |
| `given_very_short_timeout_when_lock_contended_then_fails_quickly` | Lock is held | Acquisition with 1ms timeout | Fails within reasonable time bound |

### 2. Concurrent Operations on Same Diagram

| Test Case | Given | When | Then |
|-----------|-------|------|------|
| `given_sequential_operations_when_same_diagram_then_both_succeed` | First operation completes | Second operation on same diagram | Both operations succeed and are serialized |
| `given_queued_mutations_when_flushed_then_all_applied_in_order` | Multiple mutations queued | Flush queue is called | All mutations applied in FIFO order |
| `given_rapid_sequential_locks_when_same_diagram_then_no_race_conditions` | Lock acquired and released rapidly | Multiple sequential operations | No data corruption or race conditions |
| `given_interleaved_operations_when_multiple_diagrams_then_isolated` | Operations on diagram A and B | Both execute concurrently | Each diagram maintains isolation |

### 3. Lock Release Failures

| Test Case | Given | When | Then |
|-----------|-------|------|------|
| `given_lock_when_dropped_then_automatically_released` | Lock goes out of scope | Destructor runs | Lock file is released and can be reacquired |
| `given_lock_when_release_called_then_freed` | Lock is held | `release()` is called | Lock is freed and returns Ok |
| `given_released_lock_when_release_again_then_handles_gracefully` | Lock already released | `release()` called again | Returns Ok without error (idempotent) |
| `given_lock_file_deleted_when_release_then_handles_error` | Lock file deleted externally | Release attempted | Returns appropriate error or handles gracefully |
| `given_panic_during_operation_when_lock_held_then_released_on_unwind` | Panic occurs | During locked operation | Lock is released via Drop implementation |

### 4. Queue Overflow and Resource Management

| Test Case | Given | When | Then |
|-----------|-------|------|------|
| `given_many_queued_mutations_when_counted_then_accurate` | 1000 mutations queued | Queue length queried | Returns accurate count |
| `given_large_queue_when_flushed_then_all_processed` | 100 mutations queued | Flush queue | All mutations processed without memory issues |
| `given_queue_when_cleared_then_empty` | Mutations queued | Clear queue called | Queue is empty, length is 0 |
| `given_multiple_diagrams_with_queues_when_flushed_then_isolated` | Diagrams A and B have queued mutations | Flush A only | Only A's mutations processed, B's remain |

### 5. Cross-Process Locking

| Test Case | Given | When | Then |
|-----------|-------|------|------|
| `given_lock_file_exists_when_check_locked_then_returns_true` | Lock file exists and is locked | `is_locked()` called | Returns true |
| `given_no_lock_file_when_check_locked_then_returns_false` | No lock file exists | `is_locked()` called | Returns false |
| `given_stale_lock_file_when_acquired_then_overwrites` | Old lock file exists (unlocked) | New lock acquired | Successfully acquires lock |

### 6. Edge Cases and Boundary Conditions

| Test Case | Given | When | Then |
|-----------|-------|------|------|
| `given_empty_diagram_id_when_operations_then_handles_gracefully` | Empty string diagram ID | Operations attempted | Returns appropriate error or handles |
| `given_special_chars_in_diagram_id_when_locking_then_sanitized` | Diagram ID with special characters | Lock file created | Creates valid file path |
| `given_very_long_diagram_id_when_locking_then_succeeds` | Diagram ID with 1000 characters | Lock acquired | Successfully creates lock file |
| `given_nonexistent_directory_when_locking_then_creates_directory` | Lock directory doesn't exist | Lock acquired | Directory is created automatically |

## Implementation Requirements

1. **Location**: Tests should be added to:
   - `diagram_tool/src/locking/manager.rs` in the `#[cfg(test)] mod tests` block
   - `diagram_tool/src/locking/file_lock.rs` in the `#[cfg(test)] mod tests` block
   - Optionally, a new integration test file `diagram_tool/tests/concurrent_access_tests.rs`

2. **Naming Convention**: All tests must follow `given_X_when_Y_then_Z` BDD naming pattern.

3. **Test Isolation**: Each test must use `tempfile::TempDir` for isolated file system operations.

4. **No Unwrap/Expect**: Tests must not use `.unwrap()` or `.expect()` - use `assert!` on Result::is_ok/is_err or pattern match.

5. **Concurrency Testing**: For actual concurrent scenarios, use `std::thread` and `std::sync::Barrier` where appropriate, but focus on serialized operations for deterministic testing.

6. **Performance Bounds**: Timeout-related tests should have explicit time assertions using `std::time::Instant`.

## Acceptance Criteria

- [ ] All 25+ test cases implemented
- [ ] All tests pass with `cargo test --package diagram_tool`
- [ ] No new clippy warnings introduced
- [ ] Test coverage of locking modules increases
- [ ] Moon validation passes (`moon run :test`)
- [ ] Tests verify both success and error paths
- [ ] No actual concurrent threads in tests (use serialized simulation for determinism)

## Out of Scope

- Actual multi-threaded concurrent execution (focus on lock behavior simulation)
- Network-based distributed locking
- Database-backed lock management
- Async/await lock patterns
- Performance benchmarks beyond timeout validation
