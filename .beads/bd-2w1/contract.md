# Contract: bd-2w1 - import: complete rollback matrix

## bead_id: bd-2w1
## bead_title: import: complete rollback matrix
## phase: p0
## updated_at: 2026-02-28T22:49:04Z

## ears_requirements
- **ubiquitous**: THE SYSTEM SHALL treat import as an atomic state transition.
- **event_driven**: 
  - WHEN import payload is invalid or chooser is cancelled, SHALL: THE SYSTEM SHALL preserve pre-import document and history.
- **unwanted**:
  - IF import fails, SHALL NOT: THE SYSTEM SHALL NOT consume undo history or clear selection (because: failure paths must be side-effect free)

## contracts
### preconditions
- auth_required: false
- required_inputs: []
- system_state: apply_import_contents transition helper exists

### postconditions
- state_changes:
  - Rollback tests pass for malformed, schema-invalid, and cancelled payloads

### invariants
- History snapshot changes only on successful import

## implementation_tasks
### phase_1_tests_first
- Gate: gate_0_research
- Tasks:
  1. Add remaining import rollback assertions (parallel_group: tests)

### phase_2_implementation
- Gate: gate_1_tests
- Tasks:
  1. Fix any non-atomic import assignment paths (done_when: Tests pass)

## Context Files to Read
- diagram_tool/src/ui/toolbar/persistence.rs
- diagram_tool/e2e/diagram.panels-persistence.spec.ts

## Note
- Focus on unit tests only
- DO NOT run E2E tests
