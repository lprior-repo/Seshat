# Contract: bd-39r - verification-harness

bead_id: bd-39r
bead_title: verification-harness: add replay fuzz and crash-recovery regression suite
phase: p0
updated_at: 2026-03-01T19:10:00Z

## Overview

Add replay fuzz and crash-recovery regression suite for verification.

## Preconditions

- SQLite connection is open with WAL enabled and synchronous FULL
- Rust Contract Signature: `fn run_replay_determinism_suite(seed: u64) -> Result<TestReport, VerifyError>`
- Rust Error Contract: `enum VerifyError { TestFailure, Io, Sqlite, Timeout }`

## Postconditions

- Rust Postcondition Signature: `fn run_crash_recovery_scenario(db_path: &Path) -> Result<TestReport, VerifyError>`
- Accepted operations increment revision monotonically by exactly one
- Rejected operations return structured error codes without side effects

## Invariants

- Event log remains append-only and replay deterministic
- Idempotent operation IDs never produce duplicate durable mutations
- Human-authored operations keep priority over conflicting AI operations

## Implementation Tasks

### Phase 2: Implementation
- Add replay fuzz cases for random operation streams with seeded reproducibility
- Add crash simulation tests around append and snapshot boundaries

### Phase 4: Verification
- Run moon run :ci
