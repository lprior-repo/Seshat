bead_id: bd-8sr
bead_title: release-validation: execute and report hardening config checks
phase: p0
updated_at: 2026-03-01T00:37:30Z

# Contract: release-validation

## Preconditions
- Moon CLI and planner script are installed in environment
- Normalized hardening tasks are present in moon.yml

## Postconditions
- Validation commands for task resolution and planning session state are executed
- Report includes whether beads were created and the resulting bead identifiers when available

## Acceptance Criteria
1. Moon task query executes successfully
2. ci-hardening task resolves all subtasks
3. Check, test, clippy tasks can be executed
4. Bead creation/d status available

## Validation Commands to Run
1. moon query tasks - Verify all tasks resolve
2. moon run :check - Verify cargo check works
3. moon run :test - Verify tests run (may have expected failures)
