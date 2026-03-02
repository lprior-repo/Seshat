# Contract: bd-2b3 - edge-case-bdd-tests-projection-replay

bead_id: bd-2b3
bead_title: edge-case-bdd-tests-projection-replay: Add BDD tests for projection replay edge cases
phase: p0
updated_at: 2026-03-01T23:00:00Z

## Overview

Add comprehensive BDD-style tests for projection replay edge cases covering error conditions and boundary scenarios.

## Preconditions

- Projection replay module exists at `diagram_tool/src/models/projection.rs`
- `ReplayError` enum defines error types: `InvalidEvent`, `InvariantViolation`, `UnsupportedVersion`, `CycleViolation`
- `replay_events` and `apply_event` functions are implemented

## Postconditions

- 25+ BDD tests added covering all edge cases
- All tests pass
- Tests follow BDD naming convention: `bdd_given_<condition>_when_<action>_then_<result>`

## Edge Cases Covered

### Empty Event Streams
- Empty event stream returns empty projection
- Empty stream from non-empty initial state preserves state

### Non-Sequential Revisions
- Revision gaps detected and reported as InvariantViolation
- Non-monotonic revisions detected
- Wrong start revision detected

### Duplicate Operation IDs
- Duplicate op_id returns InvariantViolation
- Pre-existing op_id in author_priority detected

### CycleViolation
- Cycle-creating edges with Deny policy return CycleViolation
- Self-loops detected as cycles
- Complex cycles in larger graphs detected
- Allow policy permits cycles

### InvariantViolation
- Duplicate node ID
- Edge to nonexistent source/target
- Node move on nonexistent node
- Edge disconnect on nonexistent edge
- Node delete on nonexistent node
- Duplicate edge ID

### Revision Increment
- Successful operations increment revision by exactly one
- Multiple operations increment revision sequentially
- Failed operations leave state unchanged (atomicity)

### Determinism
- Same events produce identical projections across multiple replays
- Same events produce identical hashes across multiple computations

### Author Priority
- Human and AI operations correctly tracked
- Large event streams track all priorities

### Error Message Quality
- Error messages are descriptive and actionable

## Test Location

`diagram_tool/src/models/projection.rs` in the `tests` module (lines 3658+)

## Acceptance Criteria

- [x] All 25 BDD tests implemented
- [x] All tests pass
- [x] Tests follow naming convention
- [x] Tests cover all edge cases listed in issue description

## Verification

```bash
cargo test --bin diagram_tool projection::tests::bdd_
```

Expected: 25 tests pass

## Implementation Notes

Tests are organized into logical sections:
1. Empty Event Stream Edge Cases
2. Non-Sequential Revision Edge Cases
3. Duplicate Operation ID Edge Cases
4. CycleViolation Edge Cases
5. InvariantViolation Edge Cases
6. Revision Increment Edge Cases
7. Determinism Edge Cases
8. Author Priority Edge Cases
9. Error Message Quality Tests
