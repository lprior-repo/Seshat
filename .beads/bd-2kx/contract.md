bead_id: bd-2kx
bead_title: tests: Implement CAM viewport tests 2/3
phase: p0
updated_at: 2026-03-02T00:03:00Z

# Contract: CAM Viewport Tests 2/3

## Scope

Add 5 viewport tests to `/home/lewis/src/seshat/diagram_tool/e2e/diagram.viewport-cam.spec.ts`:

### Test Specifications

| # | Test Name | Description |
|---|-----------|-------------|
| 1 | `spacebar + drag pans viewport without selecting nodes` | Pan via spacebar+drag; verify camera moves without selecting nodes |
| 2 | `edge scrolling during drag` | Drag node near canvas edge; verify viewport scrolls to reveal more space |
| 3 | `zoom out clamps at minimum 10%` | Zoom out beyond minimum; verify clamped at ZOOM_MIN (10%) |
| 4 | `zoom in clamps at maximum 400%` | Zoom in beyond maximum; verify clamped at ZOOM_MAX (400%) |
| 5 | `world-to-screen remains consistent at extreme coordinates` | Transform world coords at extreme values; verify consistent results |
| 6 | `fit to content with padding` | Auto-fit content to viewport; verify appropriate padding around content |

## Preconditions

- `diagram_tool/e2e/diagram.viewport-cam.spec.ts` exists with existing tests
- Playwright test infrastructure is operational
- `helpers.ts` provides required utilities: `freshStart`, `runEffect`, `runEffectsSequential`, `trapPageErrors`, `zoomPercent`, etc.

## Postconditions

- All specified tests pass: `npx playwright test diagram.viewport-cam.spec.ts`
- Tests use `@baseline` tag convention
- Tests follow existing patterns in the file
- No page errors during test execution

## Invariants

- `ZOOM_MIN = 10` and `ZOOM_MAX = 400` constants
- All tests use `freshStart()` for isolation
- All tests trap page errors and assert empty error array

## Acceptance Criteria

1. `moon run :test` passes (including Playwright tests)
2. `moon run :ci` passes
3. Tests are deterministic and reproducible
4. Tests follow existing naming and structure conventions
