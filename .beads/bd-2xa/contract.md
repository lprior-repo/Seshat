# Contract: bd-2xa - snapshot-checkpoint

bead_id: bd-2xa
bead_title: snapshot-checkpoint: implement snapshot write and tail replay boot
phase: p0
updated_at: 2026-03-01T19:02:45Z

## Overview

Implement snapshot write and tail replay boot for efficient startup.

## Preconditions

- SQLite connection is open with WAL enabled and synchronous FULL
- Rust Contract Signature: `fn write_snapshot(conn: &mut Connection, projection: &DiagramProjection) -> Result<SnapshotMeta, SnapshotError>`
- Rust Error Contract: `enum SnapshotError { SnapshotStale, Serialization, Sqlite, Replay }`

## Postconditions

- Rust Postcondition Signature: `fn load_projection(conn: &Connection) -> Result<DiagramProjection, SnapshotError>`
- Accepted operations increment revision monotonically by exactly one
- Rejected operations return structured error codes without side effects

## Invariants

- Event log remains append-only and replay deterministic
- Idempotent operation IDs never produce duplicate durable mutations
- Human-authored operations keep priority over conflicting AI operations

## System Properties

- auth_required: false
- required_inputs: []

## Acceptance Criteria

### Happy Path Tests
- test_happy_path: Given Valid inputs, when User executes command, then Exit code is 0, Output is correct

### Error Path Tests
- test_error_path: Given Invalid inputs, when User executes command, then Exit code is non-zero, Error message is clear

## Implementation Tasks

### Phase 0: Research
- Read existing model and CLI wiring before writing tests

### Phase 1: Tests First
- Write failing tests for happy and error paths

### Phase 2: Implementation
- Persist snapshot with revision marker in independent transaction
- On startup load latest snapshot and replay events greater than snapshot revision

### Phase 4: Verification
- Run moon run :ci
