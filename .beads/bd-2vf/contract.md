# Contract: bd-2vf - viewport: harden scroll embedding calibration

## bead_id: bd-2vf
## bead_title: viewport: harden scroll embedding calibration
## phase: p0
## updated_at: 2026-02-28T22:35:50Z

## ears_requirements
- **ubiquitous**: THE SYSTEM SHALL keep world-to-screen calibration aligned after supported scroll events.
- **event_driven**: 
  - WHEN ancestor or page scroll occurs, SHALL: THE SYSTEM SHALL update canvas origin before next pointer interaction.
- **unwanted**:
  - IF user scrolls before interacting with canvas, SHALL NOT: THE SYSTEM SHALL NOT require a canvas action to refresh offsets (because: stale offset causes misplaced edits)

## contracts
### preconditions
- auth_required: false
- required_inputs: []
- system_state: Embedding regression specs exist

### postconditions
- state_changes:
  - All embedding scroll offset baseline tests pass

### invariants
- Hit-testing remains aligned after scroll plus zoom

## implementation_tasks
### phase_1_tests_first
- Gate: gate_0_research
- Tasks:
  1. Add and complete offset regression scenarios (parallel_group: tests)

### phase_2_implementation
- Gate: gate_1_tests
- Tasks:
  1. Fix origin update ordering if failing (done_when: Tests pass)

## Context Files to Read
- diagram_tool/e2e/diagram.embedding-scroll-offset.spec.ts
- diagram_tool/src/ui/canvas.rs

## Note
- Focus on unit tests only
- DO NOT run E2E tests
