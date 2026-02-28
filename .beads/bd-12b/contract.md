bead_id: bd-12b
bead_title: moon-config: normalize hardening task graph and aliases
phase: p0
updated_at: 2026-02-28T21:58:51Z

# Contract: moon-config normalization

## Preconditions
- moon.yml exists at repository root
- Moon CLI resolves project tasks from root configuration

## Postconditions
- moon.yml defines explicit e2e-smoke and e2e-full tasks
- ci-hardening task exists and references the required hardening sequence

## Invariants
- Hardening task order remains check -> test -> clippy -> e2e-smoke -> e2e-full
- No feature behavior changes are introduced by task normalization

## Acceptance Criteria
1. moon.yml defines explicit e2e-smoke task
2. moon.yml defines explicit e2e-full task
3. ci-hardening task exists and runs the full sequence
4. Task aliases match documented hardening pipeline
