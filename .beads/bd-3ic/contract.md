# Contract: bd-3ic - Fix failing edge routing tests

## Bead Metadata
- bead_id: bd-3ic
- bead_title: tests: Fix failing edge routing tests
- phase: p0
- updated_at: 2026-03-01T16:02:12Z

## Contract Summary
Fix 5 failing tests in diagram_tool/e2e/diagram.edges-and-routing.spec.ts: edge creation, hit-selection, thin edge zoom, endpoint clicks. Tests: EDG-001, EDG-003, EDG-033, EDG-034.

## Acceptance Criteria
1. All 5 edge routing tests pass (EDG-001, EDG-003, EDG-033, EDG-034, plus one unlisted)
2. No regression in other edge-related tests
3. Tests follow existing test patterns in the codebase

## Implementation Notes
- Use existing test patterns from other passing tests
- Ensure tests are deterministic and don't rely on timing
- Follow zero-unwrap law: use Result<T, Error> patterns

## Current Status
**BLOCKED**: The test infrastructure is broken due to WASM build failure. The dx serve command fails with:
- `sqlite3/sqlite3.c: fatal error: 'stdio.h' file not found` for wasm32 target
- Backend connection fails because the fullstack server can't start properly

This is an infrastructure issue requiring WASM cross-compilation toolchain setup, not a test logic issue.
