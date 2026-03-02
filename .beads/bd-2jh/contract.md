bead_id: bd-2jh
bead_title: tests: Implement CAM viewport tests 3/3
phase: p0
updated_at: 2026-03-02T00:40:00Z

# Contract: CAM Viewport Tests 3/3

## Scope

Add 5 viewport tests to `/home/lewis/src/seshat/diagram_tool/e2e/diagram.viewport-cam.spec.ts`:

### Test Specifications

| # | Test Name | Description |
|---|-----------|-------------|
| 1 | `canvas embedded in scrollable parent handles coordinate offset` | Canvas in scrollable harness; verify world-to-screen transforms account for scroll offset |
| 2 | `viewport recalculates after DPR change` | Simulate devicePixelRatio change; verify zoom and coordinates remain consistent |
| 3 | `context menu focus loss mid-drag does not corrupt selection` | Open context menu during drag, dismiss it; verify selection state is intact |
| 4 | `auto-save preserves camera position without stutter` | Trigger auto-save cycle; verify camera position does not jump or reset unexpectedly |
| 5 | `pan inertia decays smoothly to stop` | Pan with momentum; verify camera gradually decelerates to a stop (if inertia implemented) or verify pan stops cleanly on mouse up |

## Preconditions

- `diagram_tool/e2e/diagram.viewport-cam.spec.ts` exists with existing tests
- Playwright test infrastructure is operational
- `helpers.ts` provides required utilities: `freshStart`, `runEffect`, `runEffectsSequential`, `trapPageErrors`, `zoomPercent`, `mountScrollableHarness`, `scrollHarnessTo`, `waitForNoRebuildOverlay`, `nodeCount`, `selectedCount`, `createTextNode`

## Postconditions

- All specified tests pass: `npx playwright test diagram.viewport-cam.spec.ts`
- Tests use `@baseline` tag convention
- Tests follow existing patterns in the file
- No page errors during test execution

## Invariants

- `ZOOM_MIN = 10` and `ZOOM_MAX = 400` constants (existing)
- All tests use `freshStart()` for isolation
- All tests trap page errors and assert empty error array

## Acceptance Criteria

1. `moon run :test` passes (including Playwright tests)
2. `moon run :ci` passes
3. Tests are deterministic and reproducible
4. Tests follow existing naming and structure conventions
5. Each test validates one specific viewport behavior
