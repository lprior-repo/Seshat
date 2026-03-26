# E2E Test Results: ThemeToggle

**Date:** Thu Mar 26 2026
**Test file:** `diagram_tool/e2e/theme-toggle.spec.ts`
**Playwright version:** 1.58.2
**Runner:** `npx playwright test diagram_tool/e2e/theme-toggle.spec.ts --reporter=list`

## Summary

| Metric | Count |
|--------|-------|
| Passed | 6 |
| Failed | 3 |
| Flaky  | 3 |
| Skipped | 0 |
| Total  | 12 (2 projects x 6 tests, baseline has 1 retry) |

## Environment

- **Dev server:** `dx serve --platform web --port 8081 --open false --watch false --hot-reload false --interactive false` (started manually; `moon run :serve-e2e` webServer config exited early)
- **Projects run:** `e2e-smoke` (retries: 0, workers: 4) and `baseline` (retries: 1, workers: 4)
- **Duration:** 49.2s

## Commands Run

```bash
# Dev server (manual, since moon run :serve-e2e failed)
cd diagram_tool && dx serve --platform web --port 8081 --open false --watch false --hot-reload false --interactive false > /tmp/dx-serve.log 2>&1 &

# Tests
npx playwright test diagram_tool/e2e/theme-toggle.spec.ts --reporter=list
```

## Passed Tests (6)

1. `[e2e-smoke]` toggle shows correct mode after cycling to each variant (4.1s)
2. `[e2e-smoke]` clicking toggle cycles through all 4 modes (4.1s)
3. `[e2e-smoke]` localStorage is updated on each toggle click (706ms)
4. `[e2e-smoke]` survives rapid toggle clicks (766ms)
5. `[baseline]` localStorage is updated on each toggle click (889ms)
6. `[baseline]` survives rapid toggle clicks (680ms)

## Failed Tests (3)

### Failure 1: `[e2e-smoke]` theme toggle button renders on canvas

- **Error:** `Test timeout of 45000ms exceeded`
- **Root cause:** `page.goto("http://127.0.0.1:8081/")` timed out waiting for `domcontentloaded`
- **Location:** `helpers.ts:22` (inside `runEffect` wrapper)
- **Note:** Passed on retry in `baseline` project

### Failure 2: `[e2e-smoke]` persisted theme is loaded on page reload

- **Error:** `Test timeout of 45000ms exceeded`
- **Root cause:** Same `page.goto` timeout as Failure 1
- **Location:** `helpers.ts:22`
- **Note:** Did NOT pass on retry — see Failure 3

### Failure 3: `[baseline]` persisted theme is loaded on page reload (both attempts)

- **Error:** `expect(received).toBe(expected)` — Expected `"Dark"`, Received `"System"`
- **Root cause:** After `page.reload({ waitUntil: "domcontentloaded" })` and `waitForNoRebuildOverlay`, the theme label polls to `"System"` instead of the previously-set `"Dark"`. localStorage persistence appears broken on reload — the `"Dark"` value set before reload is not restored.
- **Location:** `theme-toggle.spec.ts:79-81`
- **Artifacts:** screenshots, videos, traces in `test-results/`

## Flaky Tests (3)

These failed on first attempt but passed on retry (`baseline` project, retries: 1):

1. `[baseline]` theme toggle button renders on canvas (timeout on first, passed retry at 688ms)
2. `[baseline]` clicking toggle cycles through all 4 modes (timeout on first, passed retry at 881ms)
3. `[baseline]` toggle shows correct mode after cycling to each variant (timeout on first, passed retry at 802ms)

**Flaky root cause:** `page.goto` timeout — likely WASM compilation/ hydration race on cold start. The `dx serve` dev server may not be fully ready when the first wave of tests hits it, even though `curl` returns 200 (HTML shell loads, but WASM asset may still be compiling).

## Analysis

1. **`page.goto` timeouts (5 of 6 failures):** The `dx serve` dev server returns HTTP 200 for the HTML shell, but the WASM module may still be compiling on first access. When multiple workers (4) hit the server simultaneously, the first requests may hang waiting for WASM hydration. This is a dev-server warmup issue, not a code bug.

2. **`persisted theme is loaded on page reload` (real bug):** After setting theme to "Dark" and reloading the page, the theme resets to "System" instead of restoring "Dark". This suggests the `localStorage.getItem("diagram_tool.theme_mode")` value is either:
   - Not being read on app initialization
   - Being cleared during the WASM re-initialization on reload
   - The hydration path does not check localStorage before rendering

## Artifacts

- Screenshots, videos, and traces saved to `test-results/theme-toggle-theme-toggle-*/`
- View traces: `npx playwright show-trace test-results/<artifact-dir>/trace.zip`
