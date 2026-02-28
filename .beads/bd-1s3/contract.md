# Contract: bd-1s3 - baseline: execute and fix failing suite

## bead_id: bd-1s3
## bead_title: baseline: execute and fix failing suite
## phase: p0
## updated_at: 2026-02-28T22:19:35Z

## ears_requirements
- **ubiquitous**: THE SYSTEM SHALL keep baseline Chromium suite passing before release.
- **event_driven**: 
  - WHEN moon run :e2e-baseline is executed, SHALL: THE SYSTEM SHALL complete without failed tests.
- **unwanted**:
  - IF a baseline test fails, SHALL NOT: THE SYSTEM SHALL NOT ship without a regression test and fix (because: release quality gate requires baseline stability)

## contracts
### preconditions
- auth_required: false
- required_inputs: []
- system_state: Moon tasks for baseline E2E exist and parse

### postconditions
- state_changes:
  - Baseline project exits with success
  - Any discovered bug has regression coverage

### invariants
- No feature additions
- Fixes stay within existing behavior contracts

## implementation_tasks
### phase_1_tests_first
- Gate: gate_0_research
- Tasks:
  1. Run baseline suite and capture failing specs (parallel_group: tests)

### phase_2_implementation
- Gate: gate_1_tests
- Tasks:
  1. Apply minimal bug fixes and update tests (done_when: Tests pass)

## Context Files to Read
- playwright.config.ts
- moon.yml
- .moon/tasks.yml
- diagram_tool/e2e/*.spec.ts

## Important
- DO NOT run E2E tests (skip :e2e-smoke, :e2e-full)
- Focus on unit tests only (moon run :test)
- The pre-existing hit-test failure at low zoom is tracked separately
