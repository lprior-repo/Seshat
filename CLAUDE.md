# Seshat Project Guidelines

This project strictly follows specific engineering practices. You **must** adhere to these constraints:

## 1. Core Mandates
- **Skills**: ALWAYS invoke the `functional-rust` skill ALWAYS before starting work.
- **TDD Guard**: You MUST use strict Test-Driven Development. No implementation code without a failing test. Tests must be piped through `tdd-guard-rust` (e.g., `cargo nextest run 2>&1 | tdd-guard-rust --project-root . --passthrough`).
- **NO Subagents**: You must NEVER use subagents or autonomous execution (`go-skill`, `Task` tool). All dev work happens in this interactive session.
- **Codanna**: Always use `codanna` for semantic search.
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
4. Push to remote (`git pull --rebase`, `bd sync`, `git push`, verify status is up to date). NEVER leave work stranded locally.

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
# Install Dioxus CLI (if not present)
cargo install dioxus-cli

# Start the dev server with hot-reload
cd diagram_tool && dx serve --port 3333 --open false

# Build for production
dx bundle
```

**IMPORTANT**: Do NOT use `cargo run` for the web app. It will panic on non-wasm targets because the codebase uses `wasm-bindgen` imports. Always use `dx serve`.

**Validation commands:**
```bash
moon run :ci-source    # Full CI pipeline (fmt + clippy + tests)
moon run :test         # Run unit tests
npx playwright test    # Run E2E tests
```