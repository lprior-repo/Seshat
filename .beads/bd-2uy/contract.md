# Contract: bd-2uy - ui-sync: update toolbar and ui state sync semantics

bead_id: bd-2uy
bead_title: ui-sync: update toolbar and ui state sync semantics
phase: p0
updated_at: 2026-03-01T13:03:13Z

---

## EARS Requirements

### Ubiquitous
- THE SYSTEM SHALL emit JSONL events for every CLI command stage
- THE SYSTEM SHALL preserve a valid last-known-good diagram state

### Event-Driven
- WHEN a mutation request is accepted, THE SYSTEM SHALL run the full validation pipeline before persistence and broadcast
- WHEN validation fails, THE SYSTEM SHALL reject the mutation and return a machine-readable error code

### Unwanted
- IF concurrent updates target the same diagram revision, THE SYSTEM SHALL NOT silently overwrite one update with another (Silent overwrites lose user or AI intent and corrupt trust)

---

## Preconditions

- auth_required: false
- required_inputs: []
- system_state:
  - Input JSON parses into the expected command payload
  - Diagram identifier resolves to a known or creatable single-diagram slot

---

## Postconditions

- state_changes:
  - Command exits with deterministic exit code and JSONL finish event
  - Persisted diagram remains schema-valid and DAG-valid on success
- return_guarantees: []

---

## Invariants

- Revision is monotonic and only server-owned
- Invalid mutations never reach UI broadcast

---

## Research Requirements

### Files to Read
- diagram_tool/src/cli.rs - Existing patterns
- diagram_tool/src/backend.rs - Existing patterns
- diagram_tool/src/patch.rs - Existing patterns
- diagram_tool/src/models/document.rs - Existing patterns

### Research Questions
1. Which existing modules should host the shared mutation pipeline?
2. Where should JSONL event structs live for reuse across commands?

---

## Implementation Tasks

### Phase 0: Research
- Read current CLI and backend flow to identify insertion points
- Map existing validation and layout functions to target pipeline stages

### Phase 1: Tests First
- Write failing integration test for command JSONL format and exit code map
- Write failing test for rejection path preserving last-known-good state

### Phase 2: Implementation
- Implement ui changes for: Ensure UI save/load and live updates use the shared pipeline, apply only valid states, and expose revision/state feedback to humans.
- Add structured error-code mapping and JSONL serializer

---

## Acceptance Tests

### Happy Paths
- test_happy_path: Given Valid inputs, when User executes command, then Exit code is 0, Output is correct

### Error Paths
- test_error_path: Given Invalid inputs, when User executes command, then Exit code is non-zero, Error message is clear

---

## Verification Checkpoints

- gate_0_research: Research Gate - All research questions answered (evidence: research notes)
- gate_1_tests: Test Gate - All tests written and failing (evidence: test files exist)
- gate_2_implementation: Implementation Gate - All tests pass (evidence: CI green)
- gate_3_integration: Integration Gate - E2E tests pass (evidence: manual verification)

---

## AI Hints

- DO: Use functional patterns (map, and_then, ?), Return Result<T, Error>, READ files before modifying
- DO NOT: Use unwrap/expect, panic!/todo!/unimplemented!, modify clippy config
- Constitution: Zero unwrap law (NEVER use .unwrap or .expect), Test first (Tests MUST exist before implementation)

---

## Completion Checklist

- [ ] All acceptance tests written and passing
- [ ] All error path tests written and passing
- [ ] E2E pipeline test passing with real data
- [ ] No mocks or fake data in any test
- [ ] Implementation uses Result<T, Error> throughout
- [ ] Zero unwrap or expect calls
- [ ] moon run :ci passes
