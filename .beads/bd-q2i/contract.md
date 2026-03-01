bead_id: bd-q2i
bead_title: test-ci: run full ci-hardening and fix remaining failures
phase: p0
updated_at: 2026-03-01T20:58:00Z

# Contract: bd-q2i - test-ci: run full ci-hardening and fix remaining failures

## EARS Requirements

### Ubiquitous
- "THE SYSTEM SHALL pass moon run ci-hardening force with zero permanent failures"
- "THE SYSTEM SHALL maintain all 431 rust unit tests passing alongside e2e improvements"

### Event-Driven
- {trigger: "WHEN moon run ci-hardening is executed", shall: "THE SYSTEM SHALL run check clippy test-rust e2e-baseline e2e-seeded and e2e-stress in sequence"}
- {trigger: "WHEN any test fails after max retries", shall: "THE SYSTEM SHALL report the failure with trace video and screenshot artifacts"}

### Unwanted
- {condition: "IF e2e changes cause rust test regressions", shall_not: "THE SYSTEM SHALL NOT break any existing 431 unit tests or 8 CLI integration tests", because: "The e2e hook is additive and must not affect core logic"}

## Contracts

### Preconditions
- auth_required: false
- required_inputs: []
- system_state:
  - "All prior tasks completed successfully"
  - "Rust code compiles and passes clippy"

### Postconditions
- state_changes:
  - "ci-hardening passes end to end"
  - "No flaky tests remain in baseline suite"
- return_guarantees: []

### Invariants
- "All 431 unit tests remain passing"
- "All 8 CLI integration tests remain passing"
- "No permanent failures in ci-hardening"

## Acceptance Tests

### Happy Paths
- {name: "test_ci_hardening_passes", given: "All code compiles", when: "moon run :ci-hardening --force executes", then: ["Exit code is 0", "All checks pass"]}

### Error Paths
- {name: "test_ci_failures_are_reported", given: "Test fails in ci-hardening", when: "max retries exceeded", then: ["Failure is reported with artifacts"]}

## Implementation Tasks

### Phase 0: Research
- {task: "Run ci-hardening to identify current failures", done_when: "Failures identified and documented"}

### Phase 1: Fix Failures
- {task: "Fix any failing tests in the baseline suite", done_when: "Tests pass"}
- {task: "Fix any flaky tests causing intermittent failures", done_when: "Tests pass consistently"}

### Phase 2: Verification
- {task: "Run moon run :ci --force", done_when: "All phases pass"}
