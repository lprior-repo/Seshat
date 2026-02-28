# Contract: bd-29g - harness: stabilize selectors and deterministic waits

## bead_id: bd-29g
## bead_title: harness: stabilize selectors and deterministic waits
## phase: p0
## updated_at: 2026-02-28T21:59:22Z

## ears_requirements
- **ubiquitous**: THE SYSTEM SHALL use stable data-testid selectors for baseline E2E interactions.
- **event_driven**: 
  - WHEN baseline E2E specs query UI controls, SHALL: THE SYSTEM SHALL resolve selectors without role/text ambiguity.
- **unwanted**:
  - IF tests rely on fixed sleeps or fragile CSS selectors, SHALL NOT: THE SYSTEM SHALL NOT allow non-deterministic waits in baseline specs (because: flake and false negatives block release confidence)

## contracts
### preconditions
- auth_required: false
- required_inputs: []
- system_state: Playwright helper layer exists under diagram_tool/e2e/helpers.ts

### postconditions
- state_changes:
  - All baseline specs use helper-driven deterministic waits or explicit readiness checks

### invariants
- No waitForTimeout in baseline suite
- No XPath selectors in baseline suite

## implementation_tasks
### phase_1_tests_first (Test-First)
- Gate: gate_0_research
- Tasks:
  1. Add or adjust tests to require deterministic selectors (parallel_group: tests)

### phase_2_implementation
- Gate: gate_1_tests  
- Tasks:
  1. Patch remaining brittle selectors and timing waits (done_when: Tests pass)

## verification_checkpoints
- gate_1_tests: Test Gate - must have failing tests before implementation
- gate_2_implementation: Implementation Gate - tests must pass
