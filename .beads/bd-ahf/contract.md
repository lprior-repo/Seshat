bead_id: bd-ahf
bead_title: verify-occ-idempotency: add stale revision and duplicate op regression tests
phase: p0
updated_at: 2026-03-01T21:15:00Z

# Contract: bd-ahf - verify-occ-idempotency: add stale revision and duplicate op regression tests

## EARS Requirements

### Ubiquitous
- "THE SYSTEM SHALL use a hard-cutover rewrite with no legacy compatibility layer"

### Event-Driven
- {trigger: "WHEN a backend change is implemented", shall: "THE SYSTEM SHALL remove conflicting legacy behavior before enabling replacement behavior"}

### Unwanted
- {condition: "IF legacy and new backends coexist in execution paths", shall_not: "THE SYSTEM SHALL NOT permit dual-write or fallback migration behavior", because: "dual paths create hidden divergence and higher defect risk"}

## Contracts

### Preconditions
- auth_required: false
- required_inputs: []
- system_state:
  - "Rust Contract Signature: fn run_occ_idempotency_suite() -> Result<TestReport, VerifyError>"
  - "Rust Error Contract: enum VerifyError { AssertionFailure, Harness, Sqlite }"

### Postconditions
- state_changes:
  - "Rust Postcondition Signature: fn assert_occ_properties(report: &TestReport) -> Result<(), VerifyError>"
  - "Legacy path is deleted or unreachable by compile-time guarantees"
  - "Replacement path passes focused tests with no fallback to removed code"
- return_guarantees: []

### Invariants
- "No migration path is introduced"
- "No dual-write compatibility path exists"
- "All fallible operations use typed Result errors"

## Acceptance Tests

### Happy Paths
- {name: "test_occ_stale_revision_rejects", given: "Append with based_on_revision lower than current", when: "Operation submitted", then: ["Returns RevisionMismatch error", "No event appended"]}
- {name: "test_idempotent_duplicate_returns_success", given: "Exact duplicate op_id already exists", when: "Same operation submitted", then: ["Returns success with no-op", "No duplicate event appended"]}

### Error Paths
- {name: "test_duplicate_mismatched_payload_rejects", given: "Same op_id with different payload", when: "Operation submitted", then: ["Returns error", "No event appended"]}

## Implementation Tasks

### Phase 1: Write Tests
- {task: "Write test for stale revision rejection", done_when: "Test exists and fails"}
- {task: "Write test for idempotent duplicate handling", done_when: "Test exists and fails"}

### Phase 2: Implementation
- {task: "Implement OCC check in append path", done_when: "Tests pass"}
- {task: "Implement duplicate detection", done_when: "Tests pass"}
