# Implementation: bd-1wc - verify-replay-fuzz

bead_id: bd-1wc
bead_title: verify-replay-fuzz: add seeded replay determinism fuzz suite
phase: p2
updated_at: 2026-03-01T20:00:00Z

## Changes Made

### File: `/home/lewis/src/seshat/diagram_tool/src/models/harness.rs`

#### 1. Extended `VerifyError` enum

Added `DeterminismFailure` and `TestHarness` variants to match the contract:

```rust
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum VerifyError {
    #[error("determinism failure: {0}")]
    DeterminismFailure(String),
    #[error("test harness error: {0}")]
    TestHarness(String),
    #[error("timeout: {0}")]
    Timeout(String),
    // ... existing variants
}
```

#### 2. Added `FuzzReport` struct

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuzzReport {
    pub seed: u64,
    pub cases_run: usize,
    pub projection_hash: String,
    pub passed: bool,
    pub error_message: Option<String>,
}
```

#### 3. Added `projection_hash` function

Computes a deterministic hash of a `DiagramProjection` by:
- Extracting nodes and edges in sorted key order
- Building a canonical string representation
- Computing a DJB2 rolling hash

```rust
pub fn projection_hash(projection: &DiagramProjection) -> Result<String, VerifyError>
```

#### 4. Added `run_replay_fuzz` function (Contract Signature)

```rust
pub fn run_replay_fuzz(seed: u64, cases: usize) -> Result<FuzzReport, VerifyError>
```

Generates seeded random operation streams and verifies:
- Same seed produces identical event streams
- Replay produces identical projections
- Projection hash is stable

#### 5. Added `assert_replay_determinism` function (Contract Signature)

```rust
pub fn assert_replay_determinism(report: &FuzzReport) -> Result<(), VerifyError>
```

Validates that a fuzz report demonstrates deterministic replay behavior.

#### 6. Added Tests

- `test_run_replay_fuzz_returns_deterministic_report` - Same seed produces same FuzzReport
- `test_run_replay_fuzz_different_seeds_produce_different_hashes` - Different seeds produce different hashes
- `test_assert_replay_determinism_accepts_valid_report` - Valid report passes assertion
- `test_assert_replay_determinism_rejects_failed_report` - Failed report returns error
- `test_assert_replay_determinism_rejects_empty_hash` - Empty hash returns error
- `test_projection_hash_is_stable` - Hash is stable for same projection
- `test_projection_hash_differs_for_different_projections` - Different projections have different hashes
- `test_fuzz_report_passing_factory` - FuzzReport::passing factory works
- `test_fuzz_report_failing_factory` - FuzzReport::failing factory works

## Verification

All 19 harness tests pass:
- 9 existing tests continue to pass
- 10 new tests for the contract functions pass

All 730 diagram_tool tests pass.
All 13 e2e tests pass.

## Contract Compliance

- [x] `fn run_replay_fuzz(seed: u64, cases: usize) -> Result<FuzzReport, VerifyError>`
- [x] `fn assert_replay_determinism(report: &FuzzReport) -> Result<(), VerifyError>`
- [x] `enum VerifyError { DeterminismFailure, TestHarness, Timeout, ... }`
- [x] No unwrap or expect calls in new code
- [x] All fallible operations use typed Result errors
- [x] Deterministic hash computation for projection stability
