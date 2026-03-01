# Contract: bd-1wc - verify-replay-fuzz

bead_id: bd-1wc
bead_title: verify-replay-fuzz: add seeded replay determinism fuzz suite
phase: p0
updated_at: 2026-03-01T19:30:00Z

## Overview

Add seeded replay determinism fuzz testing suite to verify that replay is
deterministic across repeated runs with the same seed.

## Preconditions

- Existing replay infrastructure (`replay_events`, `replay_events_from`)
- Existing `SeededRng` implementation for deterministic randomness
- Rust Contract Signature: `fn run_replay_fuzz(seed: u64, cases: usize) -> Result<FuzzReport, VerifyError>`
- Rust Error Contract: `enum VerifyError { DeterminismFailure, TestHarness, Timeout }`

## Postconditions

- Rust Postcondition Signature: `fn assert_replay_determinism(report: &FuzzReport) -> Result<(), VerifyError>`
- Fuzz tests are deterministic with same seed
- Can detect non-deterministic replay
- Stable projection hash across repeated runs

## Invariants

- No migration path is introduced
- No dual-write compatibility path exists
- All fallible operations use typed Result errors
- Zero unwrap or expect calls

## Implementation Tasks

### Phase 2: Implementation
- Add `FuzzReport` struct with hash and case information
- Add `run_replay_fuzz(seed, cases)` function that generates seeded random operation streams
- Add `assert_replay_determinism(report)` function that verifies stable hashes
- Compute stable projection hash using deterministic serialization

### Phase 4: Verification
- Run moon run :ci

## Acceptance Tests

1. `test_run_replay_fuzz_returns_deterministic_report` - Same seed produces same FuzzReport
2. `test_assert_replay_determinism_accepts_valid_report` - Valid report passes assertion
3. `test_assert_replay_determinism_rejects_mismatched_hash` - Hash mismatch returns error
4. `test_projection_hash_is_stable` - Hash is stable across repeated runs
