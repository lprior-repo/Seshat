# Contract: bd-320 - verify-crash-recovery

bead_id: bd-320
bead_title: verify-crash-recovery: add append and snapshot crash boundary tests
phase: p0
updated_at: 2026-03-01T19:20:00Z

## Overview

Add crash recovery tests around append and snapshot boundaries.

## Preconditions

- Existing append and snapshot infrastructure
- Rust Contract Signature: `fn test_crash_at_boundary(boundary: CrashBoundary) -> Result<CrashTestReport, VerifyError>`

## Postconditions

- Tests verify recovery works after simulated crashes
- Tests cover append and snapshot boundaries

## Implementation Tasks

### Phase 2: Implementation
- Add crash simulation around append
- Add crash simulation around snapshot

### Phase 4: Verification
- Run moon run :ci
