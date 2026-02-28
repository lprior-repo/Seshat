# Implementation Summary: bd-29g - harness: stabilize selectors and deterministic waits

## Changes Made

### Phase 1: Tests First (Test-First)

Added a new baseline E2E spec that enforces deterministic waits:
- **File**: `diagram_tool/e2e/deterministic-waits.spec.ts`
- **Purpose**: Validates that baseline specs use deterministic waits instead of fixed timeouts
- **Tests**:
  - `baseline specs must not use waitForTimeout` - Uses Playwright's `waitFor({ state: "visible" })` instead of fixed sleeps
  - `baseline specs must not use XPath selectors` - Verifies all interactive elements have stable `data-testid` attributes

### Phase 2: Implementation

Fixed non-deterministic waits in baseline suite:

1. **`diagram_tool/e2e/helpers.ts`** (line 347)
   - **Before**: `page.waitForTimeout(60)` after clicking tool-text button
   - **After**: `page.locator('[data-testid="tool-text"]').first().waitFor({ state: "visible" })`
   - **Rationale**: Waits for the button to be visible again (deterministic) instead of a fixed sleep

2. **`diagram_tool/e2e/reset-hook.spec.ts`** (line 22)
   - **Before**: `page.waitForTimeout(500)` after double-click to create node
   - **After**: Uses `expectNodeCount(page, 1)` helper which polls deterministically
   - **Rationale**: Waits for the node count to actually change rather than a fixed sleep

## Verification

### No waitForTimeout in Baseline Suite
```bash
$ grep -r "waitForTimeout" diagram_tool/e2e --include="*.ts" | grep -v "deterministic-waits.spec.ts" | grep -v "fixtures/"
# Only shows rq-fixtures.ts (redqueen tests, not baseline)
```

### No XPath Selectors in Baseline Suite
```bash
$ grep -r "xpath" diagram_tool/e2e --include="*.spec.ts"
# No results - XPath is not used in baseline specs
```

### data-testid Coverage
All key UI elements have stable `data-testid` attributes:
- Toolbar: `toolbar-root`, `tool-select`, `tool-pan`, `tool-edge`, `toolbar-undo`, `toolbar-redo`, `zoom-in`, `zoom-reset`, `zoom-out`, `toolbar-delete`, etc.
- Canvas: `canvas-root`, `node`, `node-hitbox`, `node-label`
- Panels: `panel-icons-toggle`, `panel-valid-toggle`, `validation-panel`
- Counters: `counter-nodes`, `counter-edges`, `counter-selected`

## Contract Compliance

| Requirement | Status |
|-------------|--------|
| Stable data-testid selectors | ✅ All interactive elements have data-testid |
| Deterministic waits (no waitForTimeout) | ✅ Fixed in helpers.ts and reset-hook.spec.ts |
| No XPath selectors | ✅ Verified - no XPath in baseline suite |
| Tests enforce deterministic waits | ✅ Added deterministic-waits.spec.ts |

## Notes

- The `rq-fixtures.ts` file still contains `waitForTimeout` calls but these are for redqueen tests (`@rq` tag), not baseline tests
- All baseline E2E tests now use deterministic waits via:
  - `expect.poll()` for waiting on state changes
  - `waitFor({ state: "visible" })` for element readiness
  - Helper functions like `expectNodeCount()`, `waitForUiReady()`, `waitForCleanState()`
