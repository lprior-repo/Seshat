# Contract: bd-1yu - history: guarantee gesture atomicity

## bead_id: bd-1yu
## bead_title: history: guarantee gesture atomicity
## phase: p0
## updated_at: 2026-02-28T22:43:18Z

## ears_requirements
- **ubiquitous**: THE SYSTEM SHALL map one completed user gesture to one history entry.
- **event_driven**: 
  - WHEN duplicate pointerup or mouseup events arrive, SHALL: THE SYSTEM SHALL finalize motion at most once.
- **unwanted**:
  - IF blur splits key nudge sequence, SHALL NOT: THE SYSTEM SHALL NOT merge separate gestures into one undo step (because: undo semantics become unpredictable)

## contracts
### preconditions
- auth_required: false
- required_inputs: []
- system_state: interaction reducer finalize path is test-covered

### postconditions
- state_changes:
  - History atomicity tests pass for drag, resize, and nudge

### invariants
- Revision increments once per finalized motion

## implementation_tasks
### phase_1_tests_first
- Gate: gate_0_research
- Tasks:
  1. Add missing history atomicity regressions (parallel_group: tests)

### phase_2_implementation
- Gate: gate_1_tests
- Tasks:
  1. Fix reducer and canvas release race defects (done_when: Tests pass)

## Context Files to Read
- diagram_tool/e2e/diagram.scale-history-races.spec.ts
- diagram_tool/e2e/diagram.history-nudge-atomicity.spec.ts
- diagram_tool/src/ui/canvas/interaction_reducer.rs
- diagram_tool/src/ui/canvas.rs

## Note
- Focus on unit tests only
- DO NOT run E2E tests
