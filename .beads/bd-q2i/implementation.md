bead_id: bd-q2i
bead_title: test-ci: run full ci-hardening and fix remaining failures
phase: p2
updated_at: 2026-03-01T21:05:00Z

# Implementation: bd-q2i - test-ci: run full ci-hardening and fix remaining failures

## Summary

Fixed clippy failure that was blocking the ci-hardening pipeline.

## Changes Made

### Fixed: Unused import in harness.rs

**File**: `diagram_tool/src/models/harness.rs`

**Issue**: The import `replay_events_from` was included at the top level but only used inside `#[cfg(test)]` blocks, causing clippy to fail with `-D warnings`.

**Fix**: Removed the unused import from the top-level import statement. The function is still accessible within test modules via `use super::*;`.

**Diff**:
```rust
// Before:
use crate::models::projection::{
    replay_events, replay_events_from, DiagramProjection, EventRecord,
};

// After:
use crate::models::projection::{replay_events, DiagramProjection, EventRecord};
```

## Verification

- `cargo check` - PASSES
- `cargo test` - PASSES (730 unit tests + 13 CLI e2e tests)
- `cargo clippy` - PASSES (with strict warnings)

## Notes

- The e2e tests (e2e-smoke, e2e-full) require a running server which is not available in this environment
- The core Rust CI pipeline (check, test, clippy) now passes
