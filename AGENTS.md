# AGENTS

```jsonl
{"skill": "moon", "description": "Build system - task running, caching, CI/CD", "commands": ["moon run <task>", "moon :ci-hardening --force", "moon check", "moon test", "moon run :clippy-source", "moon run :ci-source"]}
{"skill": "functional-rust", "description": "Zero panics/unwrap/mut - Data→Calc→Actions pattern", "rules": ["No panics", "No unwrap", "No mut by default", "Result<T, E> for errors", "clippy-source pipeline for flawless source"]}
{"skill": "landing-skill", "description": "Session completion - validates quality, syncs main, closes bead", "commands": ["/land"]}
{"skill": "dioxus-wasm-constraints", "description": "Dioxus WASM Build Constraint Guard", "rules": ["NEVER include `tokio`, `mio`, `sqlx`, or `reqwest` (with default TLS) in the `wasm32-unknown-unknown` target. ALWAYS isolate server/db dependencies behind `#[cfg(not(target_arch = \"wasm32\"))]`. ALWAYS use `default = [\"web\"]` in Cargo.toml. Dioxus `fullstack` feature MUST NOT be active when building purely for web."]}
{"workflow": "jj new main → moon run :check --force → implement → jj commit → jj push → /land", "stack": "moon + functional-rust + landing-skill + dioxus-wasm-constraints"}
```

## Building & Running

**This is a Dioxus WASM application. Use `dx` CLI (NOT `cargo run`):**

```bash
moon run :serve         # Dev server (kills port first, disables sccache)
dx bundle               # Production build
```

**CI / Validation:**
```bash
moon run :ci-source    # Format + clippy + tests
npx playwright test    # E2E tests
```

### sccache Workaround

sccache (enabled via `RUSTC_WRAPPER=sccache` in `~/.zshrc`) causes build failures with Dioxus because:
- sccache's compiler detection test creates temp files with CUDA preprocessor syntax (`#if defined(__NVCC__)`)
- When system gcc tries to preprocess these files, it fails with "expected one of `!` or `[`"

The `moon run :serve` task automatically disables sccache via `env -u RUSTC_WRAPPER`. If running `dx serve` directly:

```bash
env -u RUSTC_WRAPPER dx serve --port 3333 --open false
```

To learn more about this project, refer to the core documentation in `docs/`:
- `docs/00_CODEBASE_MAP.md`: Where things live (UI, Models, CLI).
- `docs/04_DATA_CALC_ACTIONS.md`: The Functional Rust pattern (Data -> Calc -> Actions).
- `docs/06_DIOXUS_PATTERNS.md`: Frontend Dioxus 0.7 architecture and constraints.
- `docs/07_TESTING_STRATEGY.md`: Test rigor and protected files.
- `docs/10_AI_CLI_CONTRACT.md`: The JSON spec and CLI agent boundary.
- `docs/11_FEATURE_SET.md`: Frontend UI vs. Backend CLI capabilities.
- `docs/12_SINGLE_LOG_ARCHITECTURE.md`: Single-log WAL, durable execution patterns, and conflict diffs.

<!-- BEGIN BEADS INTEGRATION v:1 profile:full hash:d4f96305 -->
## Issue Tracking with bd (beads)

**IMPORTANT**: This project uses **bd (beads)** for ALL issue tracking. Do NOT use markdown TODOs, task lists, or other tracking methods.

### Why bd?

- Dependency-aware: Track blockers and relationships between issues
- Git-friendly: Dolt-powered version control with native sync
- Agent-optimized: JSON output, ready work detection, discovered-from links
- Prevents duplicate tracking systems and confusion

### Quick Start

**Check for ready work:**

```bash
bd ready --json
```

**Create new issues:**

```bash
bd create "Issue title" --description="Detailed context" -t bug|feature|task -p 0-4 --json
bd create "Issue title" --description="What this issue is about" -p 1 --deps discovered-from:bd-123 --json
```

**Claim and update:**

```bash
bd update <id> --claim --json
bd update bd-42 --priority 1 --json
```

**Complete work:**

```bash
bd close bd-42 --reason "Completed" --json
```

### Issue Types

- `bug` - Something broken
- `feature` - New functionality
- `task` - Work item (tests, docs, refactoring)
- `epic` - Large feature with subtasks
- `chore` - Maintenance (dependencies, tooling)

### Priorities

- `0` - Critical (security, data loss, broken builds)
- `1` - High (major features, important bugs)
- `2` - Medium (default, nice-to-have)
- `3` - Low (polish, optimization)
- `4` - Backlog (future ideas)

### Workflow for AI Agents

1. **Check ready work**: `bd ready` shows unblocked issues
2. **Claim your task atomically**: `bd update <id> --claim`
3. **Work on it**: Implement, test, document
4. **Discover new work?** Create linked issue:
   - `bd create "Found bug" --description="Details about what was found" -p 1 --deps discovered-from:<parent-id>`
5. **Complete**: `bd close <id> --reason "Done"`

### Auto-Sync

bd automatically syncs via Dolt:

- Each write auto-commits to Dolt history
- Use `bd dolt push`/`bd dolt pull` for remote sync
- No manual export/import needed!

### Important Rules

- ✅ Use bd for ALL task tracking
- ✅ Always use `--json` flag for programmatic use
- ✅ Link discovered work with `discovered-from` dependencies
- ✅ Check `bd ready` before asking "what should I work on?"
- ❌ Do NOT create markdown TODO lists
- ❌ Do NOT use external issue trackers
- ❌ Do NOT duplicate tracking systems

For more details, see README.md and docs/QUICKSTART.md.

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd dolt push
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds

<!-- END BEADS INTEGRATION -->

## Landing the Plane (Full Moon Landing)

**When ending a work session**, you MUST complete ALL steps below to execute a "Full Moon Landing". Work is NOT complete until the moon pipeline passes and `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - YOU MUST RUN `moon run :ci-source` to ensure strict functional-rust clippy compliance and flawless tests.
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   jj pull --rebase
   bd sync
   jj push
   jj status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds

## Test Protection (CRITICAL)

Certain test files are CONTRACT TESTS and MUST NEVER be overwritten:

### Protected Test Files

| File | Bead | Coverage |
|------|------|----------|
| `diagram_models/src/io_tests.rs` | seshat-4uc | IO-001 to IO-015 |
| `diagram_tool/src/test_infrastructure_tests.rs` | seshat-wcb | P1-P4, Q1-Q3 |
| `diagram_tool/src/geometry/**/*.rs` (tests) | seshat-pnn | GEO-001 to GEO-030 |

**RULE**: Before modifying ANY test file, check `.beads/TEST_PROTECTION.md`

```bash
# Verify protected tests exist
test -f diagram_models/src/io_tests.rs
test -f diagram_tool/src/test_infrastructure_tests.rs
grep -r "GEO-0" diagram_tool/src/geometry/ | wc -l
```

**DO NOT**: Delete, replace, merge, or "clean up" protected test files without explicit permission.
