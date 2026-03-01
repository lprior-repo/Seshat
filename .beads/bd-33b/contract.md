# Contract: bd-33b - projection-replay

bead_id: bd-33b
bead_title: projection-replay: build deterministic document projection replayer
phase: p0
updated_at: 2026-03-01T18:52:00Z

## Overview

Build a deterministic document projection replayer that replays events to produce a consistent DiagramProjection.

## Preconditions

- SQLite connection is open with WAL enabled and synchronous FULL
- Rust Contract Signature: `fn replay_events(events: &[EventRecord]) -> Result<DiagramProjection, ReplayError>`
- Rust Error Contract: `enum ReplayError { InvalidEvent, InvariantViolation, UnsupportedVersion }`

## Postconditions

- Rust Postcondition Signature: `fn apply_event(state: DiagramProjection, event: &EventRecord) -> Result<DiagramProjection, ReplayError>`
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
- test_happy_path: Given Valid inputs, when User executes command, then Exit code is 0, Output is correct

### Error Path Tests
- test_error_path: Given Invalid inputs, when User executes command, then Exit code is non-zero, Error message is clear
- test_error_path: Given Invalid inputs, when User executes command, then Exit code is non-zero, Error message is clear

## Implementation Tasks

### Phase 0: Research
- Read existing model and CLI wiring before writing tests

### Phase 1: Tests First
- Write failing tests for happy and error paths

### Phase 2: Implementation
- Build pure replay module with exhaustive op matching
- Add determinism tests comparing projection hash across repeated replays

### Phase 4: Verification
- Run moon run :ci
