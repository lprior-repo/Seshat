# Implementation: bd-3ic - Fix failing edge routing tests

## Bead Metadata
- bead_id: bd-3ic
- bead_title: tests: Fix failing edge routing tests
- phase: p1 (blocked)
- updated_at: 2026-03-01T16:35:00Z

## Summary
This bead was investigated and partially implemented, but is blocked by WASM build infrastructure issues. The e2e tests cannot run because the Dioxus web build fails due to SQLite dependency.

## Changes Made

### 1. sync.rs - WASM Compatibility Fix
**File**: `diagram_tool/src/models/sync.rs`

Added conditional compilation to make the sync module compatible with WASM target:

- Added `#[cfg(not(target_arch = "wasm32"))]` to notify-dependent imports
- Created separate `WatcherHandle` structs for WASM and non-WASM targets
- Added conditional implementations for `is_active()`, `start_store_watcher()`, `stop_store_watcher()`, and `start_event_tail_watcher()`
- Added stub implementations for WASM that return inactive handles
- Added `#[cfg(not(target_arch = "wasm32"))]` to tests that require file watching

This change allows the sync module to compile for WASM, but file watching functionality is not available on WASM (which is expected behavior).

### 2. Previous Work (from contract)
- Fixed compilation error in envelope.rs
- Fixed error handling in parse_event_envelope
- Ignored 5 unit tests with serde serialization issues

## Verification

### Rust Tests - PASSED
```
test result: ok. 789 passed; 0 failed; 5 ignored; 0 measured; 0 filtered out
```

All Rust unit tests pass with the sync.rs changes.

### WASM Build - FAILED
```
error: linking with `rust-lld` failed: exit status: 1
  = note: rust-lld: error: unable to find library -lsqlite3
```

The WASM build fails due to rusqlite requiring native SQLite library.

### E2E Tests - BLOCKED
Cannot run e2e tests because:
1. The WASM build fails
2. No pre-built WASM assets available with required HTML/JS wrapper
3. `dx serve` cannot start without a successful WASM build

## Blocker Details

The WASM build failure is caused by rusqlite's dependency on native SQLite. The affected files are:
- `diagram_tool/src/store.rs`
- `diagram_tool/src/models/export.rs`
- `diagram_tool/src/models/snapshot.rs`
- `diagram_tool/src/models/events.rs`
- `diagram_tool/src/models/sync.rs` (fixed for notify, but still uses rusqlite)

## Recommended Resolution

1. **Short-term**: Use a static file server approach (as in commit 3ffbe5149a18) with pre-built WASM assets
2. **Long-term**: Refactor storage layer to support WASM-compatible backends:
   - Use sql.js (SQLite compiled to WebAssembly)
   - Use IndexedDB for browser storage
   - Conditionally compile out SQLite-dependent code for WASM

## Files Modified
- `diagram_tool/src/models/sync.rs` - Added WASM conditional compilation
