bead_id: bd-5id
bead_title: release-gate: enforce ci hardening checklist
phase: p0
updated_at: 2026-03-01T00:42:45Z

# Contract: release-gate

## Preconditions
- moon tasks parse and execute

## Postconditions
- Documented release gate sequence is reproducible

## Invariants
- No direct cargo or dx bypass in release gate instructions

## Acceptance Criteria
1. ci-hardening task exists and runs in order: check -> test -> clippy -> e2e-smoke -> e2e-full
2. No bypass commands in release gate instructions
3. All tasks use moon run prefix (no direct cargo/npm/dx)
