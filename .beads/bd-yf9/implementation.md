bead_id: bd-yf9
bead_title: test-migrate: convert spec files to use fresh-start fixture
phase: p1
updated_at: 2026-03-01T21:30:00Z

# Implementation: E2E Test Migration to Fresh-Start Fixture

## Research Summary

### Files Analyzed
1. `diagram_tool/e2e/helpers.ts` - Contains `freshStart()` helper (lines 343-357)
2. `diagram_tool/e2e/diagram.behavior.spec.ts` - Already migrated to `freshStart()`
3. `diagram_tool/e2e/diagram.nodes-and-selection.spec.ts` - Already migrated to `freshStart()`
4. `diagram_tool/e2e/diagram.undo-redo-history.spec.ts` - Already migrated to `freshStart()`
5. `diagram_tool/e2e/deterministic-waits.spec.ts` - Uses `page.goto()` - **Analysis needed**
6. `diagram_tool/e2e/diagram.performance.spec.ts` - Uses custom `bootPerformancePage()` - **Needs migration**

### Current State

#### Already Migrated (3 files)
- `diagram.behavior.spec.ts` - Uses `freshStart(page)` in all tests
- `diagram.nodes-and-selection.spec.ts` - Uses `freshStart(page)` in all tests
- `diagram.undo-redo-history.spec.ts` - Uses `freshStart(page)` in all tests

#### Needs Analysis (1 file)
- `deterministic-waits.spec.ts` - Meta-test that validates deterministic patterns
  - Tests do not manipulate diagram state (nodes/edges)
  - Tests verify selector and wait strategy best practices
  - Using `freshStart()` would be appropriate for consistency

#### Needs Migration (1 file)
- `diagram.performance.spec.ts` - Uses custom `bootPerformancePage()` function
  - Has special initialization script for fetch mocking
  - Needs to be refactored to use `freshStart()` with the init script

### Migration Strategy

#### deterministic-waits.spec.ts
Replace:
```typescript
await page.goto("/", { waitUntil: "load" });
```

With:
```typescript
import { freshStart } from "./helpers";
await freshStart(page);
```

#### diagram.performance.spec.ts
The `bootPerformancePage` function needs refactoring to:
1. Keep the `addInitScript` for fetch mocking
2. Replace `page.goto()` + visibility waits with `freshStart()`

Approach:
```typescript
async function bootPerformancePage(page: Page): Promise<void> {
  await runEffect(() =>
    page.addInitScript(() => {
      // ... fetch mock code ...
    }),
  );
  await freshStart(page);
}
```

## Implementation Tasks

### Task 1: Migrate deterministic-waits.spec.ts
- [ ] Add `freshStart` import
- [ ] Replace `page.goto("/")` with `freshStart(page)` in both tests
- [ ] Remove redundant visibility waits (handled by freshStart)

### Task 2: Migrate diagram.performance.spec.ts
- [ ] Import `freshStart` from helpers
- [ ] Refactor `bootPerformancePage` to use `freshStart()`
- [ ] Keep `addInitScript` call before `freshStart()`
- [ ] Remove duplicate visibility waits

## Verification Steps

1. Run tests sequentially: `cd diagram_tool && npx playwright test`
2. Run tests in parallel (12 workers): `cd diagram_tool && npx playwright test --workers=12`
3. Run Moon validation gates: `moon run :quick && moon run :test && moon run :ci`

## Risk Assessment

- **Low Risk**: deterministic-waits.spec.ts - Simple replacement
- **Medium Risk**: diagram.performance.spec.ts - Custom boot sequence with init scripts

## Rollback Plan

If tests fail after migration:
1. Revert changes using `jj undo`
2. Document failure in defects.md
3. Analyze failure mode and adjust approach
