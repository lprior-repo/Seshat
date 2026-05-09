# Seshat Project Guidelines

This project strictly follows specific engineering practices. You **must** adhere to these constraints:

## 1. Core Mandates
- **Skills**: ALWAYS invoke the `functional-rust` skill ALWAYS before starting work.
- **TDD Guard**: You MUST use strict Test-Driven Development. No implementation code without a failing test. Tests must be piped through `tdd-guard-rust` (e.g., `cargo nextest run 2>&1 | tdd-guard-rust --project-root . --passthrough`).
- **NO Subagents**: You must NEVER use subagents or autonomous execution (`Task` tool). All dev work happens in this interactive session.
- **Moon & BD**: Use `moon` for all build tasks. Use `bd` (beads) for ALL issue tracking (no markdown TODOs).
- **Jujutsu (`jj`)**: Use `jj` for version control alongside `git`.

## 2. Code Review & Auditing (CRITICAL)
- Before finalizing any feature or significant refactor, you MUST perform a code review stage.
- **Black Hat Review**: Invoke the `black-hat-reviewer` skill to mercilessly verify constraints (Contract Parity, Functional Rust Big 6, Strict DDD).
- **Truth Serum**: Invoke the `truth-serum` skill to audit the code for AI hallucinations, laziness, or skipped verification steps.

## 3. Landing the Plane (Full Moon Landing)
A session is NOT complete until all these steps are done:
1. Pass the Code Review Stage (`black-hat-reviewer` & `truth-serum`).
2. Run `moon run :ci-source` and ensure it passes completely.
3. File remaining/discovered work with `bd`.
4. Push to remote (`jj pull --rebase`, `bd sync`, `jj push`, verify status is up to date). NEVER leave work stranded locally.

## 4. Documentation Map
Consult the following before writing code:
- **`docs/00_CODEBASE_MAP.md`** - Where files live.
- **`docs/04_DATA_CALC_ACTIONS.md`** - Functional Rust rules (Zero panics/unwrap/mut, `Result<T,E>`).
- **`docs/06_DIOXUS_PATTERNS.md`** - Frontend UI state management.
- **`docs/07_TESTING_STRATEGY.md`** - Testing rigor.
- **`docs/13_LESSONS_LEARNED.md`** - **CRITICAL**: Contains strict Dioxus WASM constraints and TDD rules. Read before touching Cargo configs or UI code.

## 5. Building & Running the App

**This is a Dioxus WASM application. You MUST use the Dioxus CLI:**

```bash
# Start the dev server with hot-reload (use moon to get port killing + sccache bypass)
moon run :serve

# Build for production
dx bundle
```

**IMPORTANT**: Do NOT use `cargo run` for the web app. It will panic on non-wasm targets because the codebase uses `wasm-bindgen` imports. Always use `moon run :serve` or `dx serve`.

**Validation commands:**
```bash
moon run :ci-source    # Full CI pipeline (fmt + clippy + tests)
moon run :test         # Run unit tests
npx playwright test    # Run E2E tests
```

## 6. UI Bug Debugging Workflow

When fixing Dioxus UI bugs, do not rely on source inspection alone. Get into the live app and prove the behavior in Chromium.

1. Start from a fresh app server so Playwright and the agent do not reuse stale WASM:
   ```bash
   fuser -k 8081/tcp 2>/dev/null || true
   SESHAT_BASE_PATH=/Seshat moon run :serve-e2e >/tmp/seshat-ui-debug.log 2>&1 &
   ```
2. Always use the app base path URL: `http://127.0.0.1:8081/Seshat/`. Root `/` is not the app.
3. Use the local Dioxus agent for fast live inspection:
   ```bash
   /home/lewis/src/dioxus-agent-rs/target/release/dioxus-agent-rs --url http://127.0.0.1:8081/Seshat/ --json eval "({title: document.title, ready: window.__seshatE2eReady === true, canvas: !!document.querySelector('[data-testid=\"canvas-root\"]')})"
   /home/lewis/src/dioxus-agent-rs/target/release/dioxus-agent-rs --url http://127.0.0.1:8081/Seshat/ --json screenshot /tmp/seshat-ui.png
   ```
4. Use Playwright for regressions and interactions. Prefer existing helpers in `diagram_tool/e2e/helpers.ts`: `waitForNoRebuildOverlay`, `waitForE2eReady`, `resetDocument`, `waitForCleanState`, and `/Seshat/` navigation.
5. For layout/sidebar/toolbar bugs, run the targeted spec first:
   ```bash
   rtk playwright test diagram_tool/e2e/ui_polish.spec.ts --project=e2e-smoke --reporter=list
   ```
6. For arrows/edges/canvas pointer bugs, run:
   ```bash
   rtk playwright test diagram_tool/e2e/edge_creation.spec.ts --project=e2e-smoke --reporter=list
   ```
7. If editing Tailwind classes or `diagram_tool/input.css`, regenerate the committed CSS asset:
   ```bash
   cd diagram_tool && npx --yes @tailwindcss/cli -i ./input.css -o ./assets/tailwind.css
   ```
8. After `moon run :ci-source`, restore generated `.claude/tdd-guard/data/test.json` unless the task explicitly changes guard fixtures. Do not commit incidental guard-output churn.

Minimum UI fix proof: focused Playwright spec passes, `moon run :ci-source` passes, and a Dioxus-agent eval or screenshot proves the live app is hydrated at `/Seshat/`.

### sccache Workaround (CRITICAL)

sccache (enabled via `RUSTC_WRAPPER=sccache` in `~/.zshrc`) causes build failures with Dioxus because:
- sccache's compiler detection test creates temp files with CUDA preprocessor syntax (`#if defined(__NVCC__)`)
- When system gcc tries to preprocess these files, it fails with "expected one of `!` or `[`"

**References:**
- sccache issue #2659: https://github.com/mozilla/sccache/issues/2659
- sccache issue #2238: https://github.com/mozilla/sccache/issues/2238 (explains nvcc cannot compile preprocessed input)

The `moon run :serve` task automatically disables sccache via `env -u RUSTC_WRAPPER`. If running `dx serve` directly:

```bash
env -u RUSTC_WRAPPER dx serve --port 3333 --open false
```
