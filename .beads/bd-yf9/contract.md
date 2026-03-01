bead_id: bd-yf9
bead_title: test-migrate: convert spec files to use fresh-start fixture
phase: p0
updated_at: 2026-03-01T21:29:00Z

# Contract: E2E Test Migration to Fresh-Start Fixture

## Summary
Migrate all Playwright e2e spec files to use the `freshStart(page)` helper function, ensuring complete test isolation with zero nodes, zero edges, and zero selected state before each test.

## Preconditions

### System State
- `freshStart` helper exists in `diagram_tool/e2e/helpers.ts`
- `resetDocument` and `waitForCleanState` are exported from helpers
- All spec files currently use `page.goto()` + `waitForUiReady()` pattern

### Required Inputs
- Access to all spec files in `diagram_tool/e2e/`
- Understanding of existing test patterns

## Requirements

### Ubiquitous Requirements
1. THE SYSTEM SHALL guarantee each e2e test starts with zero nodes, zero edges, and zero selected state
2. THE SYSTEM SHALL clear localStorage, sessionStorage, and cookies before each test navigation

### Event-Driven Requirements
1. WHEN a test calls `freshStart(page)`, THE SYSTEM SHALL:
   - Clear cookies
   - Clear storage (localStorage, sessionStorage)
   - Navigate to root URL
   - Wait for UI to be ready
   - Wait for hooks to complete
   - Reset document state
   - Verify clean state

2. WHEN a spec file is migrated, THE SYSTEM SHALL:
   - Replace `page.goto()` + `waitForUiReady()` with `freshStart()` in test setup
   - Maintain all existing test assertions and behaviors

### Unwanted Behaviors
1. IF a test relies on state from a previous test, THE SYSTEM SHALL NOT allow tests to depend on shared mutable state between test cases, BECAUSE state coupling causes flakiness under parallel execution.

## Postconditions

### State Changes
1. All spec files use `freshStart()` or explicit reset sequence
2. No test assumes state from a prior test
3. All `page.goto()` + `waitForUiReady()` patterns replaced with `freshStart()`

### Invariants
1. Tests pass both sequentially and with twelve parallel workers
2. Test isolation is guaranteed by `freshStart()`
3. All existing test behaviors preserved

## Files to Migrate

Based on research requirements:
1. `diagram_tool/e2e/diagram.behavior.spec.ts`
2. `diagram_tool/e2e/diagram.nodes-and-selection.spec.ts`
3. `diagram_tool/e2e/diagram.undo-redo-history.spec.ts`

## Acceptance Criteria

### Gate 0: Research
- [ ] All spec files inventoried
- [ ] Existing patterns documented
- [ ] Count of `page.goto()` + `waitForUiReady()` patterns identified

### Gate 1: Implementation
- [ ] `diagram.behavior.spec.ts` migrated as reference pattern
- [ ] Remaining spec files migrated systematically
- [ ] All tests use `freshStart()` or explicit reset

### Gate 2: Verification
- [ ] `moon run :quick` passes
- [ ] `moon run :test` passes
- [ ] `moon run :ci` passes
- [ ] Tests pass sequentially
- [ ] Tests pass with 12 parallel workers

## Verification Commands

```bash
# Verify migration completeness
grep -r "page.goto" diagram_tool/e2e/*.spec.ts | wc -l  # Should be 0 or minimal

# Run tests sequentially
cd diagram_tool && npx playwright test

# Run tests in parallel (12 workers)
cd diagram_tool && npx playwright test --workers=12

# Moon validation gates
moon run :quick
moon run :test
moon run :ci
```

## Implementation Notes

1. Read each spec file before modifying
2. Identify all `page.goto()` + `waitForUiReady()` patterns
3. Replace with single `freshStart(page)` call
4. Preserve all existing test logic and assertions
5. Run tests after each file migration
6. Document any edge cases or special handling required
