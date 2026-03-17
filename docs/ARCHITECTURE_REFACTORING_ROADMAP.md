# Seshat CUPID Refactoring Roadmap

**Created:** 2026-03-16
**Status:** Planning Complete
**Total Beads:** 21 (6 Phase 1, 7 Phase 2, 8 Phase 3)

---

## Overview

This roadmap transforms Seshat into a CUPID-compliant, Scott Wlaschin-style functional Rust codebase. Each bead follows the enhanced template with contracts, verification gates, and explicit dependencies.

---

## Phase 1: Functional Rust Compliance (Critical Priority 0)

### P1-001: Eliminate unwrap/expect in mutation/pipeline.rs
- **Type:** bug
- **Priority:** 0
- **Effort:** 30min
- **File:** `diagram_tool/src/mutation/pipeline.rs` (1321 lines)
- **Problem:** Contains `.unwrap()` calls violating zero-panic policy
- **Contract:**
  - Pre: Pipeline operations use Result<T, E>
  - Post: Zero unwrap/expect/panic in production paths
  - Inv: All fallible operations return Result
- **Acceptance:**
  - `rg "\.unwrap\(" mutation/pipeline.rs` returns empty
  - `rg "\.expect\(" mutation/pipeline.rs` returns empty
  - `moon run :clippy-source` passes

### P1-002: Eliminate unwrap/expect in store_sqlx.rs
- **Type:** bug
- **Priority:** 0
- **Effort:** 1hr
- **File:** `diagram_tool/src/store_sqlx.rs` (2471 lines)
- **Problem:** Contains `.unwrap()` and `.expect()` in async store
- **Contract:**
  - Pre: Async store functions handle sqlx::Error
  - Post: All errors propagated via Result<T, StoreError>
  - Inv: Database errors are recoverable
- **Acceptance:**
  - All database operations return Result
  - Error messages include context for debugging
  - `moon run :clippy-source` passes

### P1-003: Eliminate unwrap/expect in backend.rs
- **Type:** bug
- **Priority:** 0
- **Effort:** 30min
- **File:** `diagram_tool/src/backend.rs`
- **Problem:** Backend initialization contains panicking code
- **Contract:**
  - Pre: Backend startup is fallible
  - Post: Backend::new() returns Result<Backend, BackendError>
  - Inv: Server startup failures are graceful
- **Acceptance:**
  - Backend::new() returns Result
  - Server binds or returns error (never panics)

### P1-004: Decompose interaction_reducer.rs (Part 1)
- **Type:** refactor
- **Priority:** 0
- **Effort:** 2hr
- **File:** `diagram_tool/src/ui/canvas/interaction_reducer.rs` (3274 lines)
- **Target Structure:**
  ```
  ui/canvas/interaction/
  ├── mod.rs          (~100 lines - re-exports)
  ├── mode.rs         (~200 lines - InteractionMode enum)
  ├── commit.rs       (~400 lines - commit_inline_edit, finalize)
  ├── reducer.rs      (~600 lines - pure state transitions)
  └── geometry.rs     (~300 lines - within, resize_target_ids)
  ```
- **Contract:**
  - Pre: interaction_reducer.rs compiles
  - Post: 5 files all <300 lines, same public API
  - Inv: No behavioral changes during refactoring
- **Dependencies:** None (can start immediately)

### P1-005: Decompose canvas.rs (Part 1)
- **Type:** refactor
- **Priority:** 0
- **Effort:** 3hr
- **File:** `diagram_tool/src/ui/canvas.rs` (3510 lines)
- **Target Structure:**
  ```
  ui/canvas/
  ├── mod.rs          (~150 lines - re-exports, Canvas component)
  ├── view.rs         (~500 lines - rendering)
  ├── input.rs        (~400 lines - event handling)
  ├── selection.rs    (~300 lines - selection logic)
  └── [existing submodules]
  ```
- **Contract:**
  - Pre: canvas.rs compiles
  - Post: Main canvas.rs <300 lines, submodules handle concerns
  - Inv: Canvas renders identically after refactor
- **Dependencies:** P1-004 (interaction decomposition first)

### P1-006: Extract pure functions from DiagramDocument
- **Type:** refactor
- **Priority:** 0
- **Effort:** 2hr
- **Files:** `diagram_tool/src/models/document/mod.rs`
- **Problem:** `select_marquee()` mixes calculation with mutation
- **Target:** Extract to `core/selection.rs`
- **Contract:**
  - Pre: DiagramDocument has methods with side effects
  - Post: Pure functions in core/, Document has only data
  - Inv: select_marquee logic unchanged, just relocated
- **Dependencies:** None

---

## Phase 2: Architectural Cleanup (Priority 1)

### P2-001: Decompose store_sqlx.rs
- **Type:** refactor
- **Priority:** 1
- **Effort:** 3hr
- **File:** `diagram_tool/src/store_sqlx.rs` (2471 lines)
- **Target Structure:** Match `store_async/` pattern
  ```
  store_sync/
  ├── mod.rs          (~50 lines)
  ├── bootstrap.rs    (~300 lines - pool creation, pragmas)
  ├── append.rs       (~400 lines - append operations)
  ├── fetch.rs        (~400 lines - query operations)
  ├── error.rs        (~100 lines - StoreError)
  └── types.rs        (~150 lines - value types)
  ```
- **Contract:**
  - Pre: store_sqlx.rs is monolithic
  - Post: 6 files all <300 lines
  - Inv: Same public API, same behavior
- **Dependencies:** P1-002 (unwrap elimination first)

### P2-002: Consolidate envelope and projection event systems
- **Type:** refactor
- **Priority:** 1
- **Effort:** 4hr
- **Files:**
  - `diagram_tool/src/models/envelope.rs`
  - `diagram_tool/src/models/projection/types.rs`
- **Problem:** EventEnvelope vs EventRecord duplication
- **Target:** Single unified event type with clear conversion
- **Contract:**
  - Pre: Two event types exist
  - Post: One canonical event type, conversion functions
  - Inv: No data loss in consolidation
- **Dependencies:** Phase 1 complete

### P2-003: Extract UI dispatch layer from app.rs
- **Type:** refactor
- **Priority:** 1
- **Effort:** 2hr
- **File:** `diagram_tool/src/app.rs` (640 lines)
- **Target:**
  ```
  ui/
  ├── app.rs          (~300 lines - main component)
  └── dispatch/
      ├── mod.rs      (~100 lines)
      ├── db.rs       (~150 lines - db coroutine)
      └── sync.rs     (~150 lines - conflict handling)
  ```
- **Contract:**
  - Pre: app.rs has dispatch logic mixed with rendering
  - Post: Dispatch logic in separate module
  - Inv: Same app behavior
- **Dependencies:** P1-005

### P2-004: Standardize error types across layers
- **Type:** refactor
- **Priority:** 1
- **Effort:** 2hr
- **Files:** All error.rs files
- **Problem:** Inconsistent error naming and structure
- **Target:**
  - `MutationError` → domain layer
  - `StoreError` → persistence layer
  - `UiError` → presentation layer
  - Clear conversion traits between layers
- **Contract:**
  - Pre: Errors use inconsistent patterns
  - Post: Layered error taxonomy
  - Inv: All errors convertible to CLI error codes
- **Dependencies:** Phase 1 complete

### P2-005: Move layout calculations to core/
- **Type:** refactor
- **Priority:** 1
- **Effort:** 2hr
- **Files:**
  - `diagram_tool/src/layout/` → `diagram_tool/src/core/layout/`
- **Problem:** Layout is calculation but lives outside core
- **Target:** Move layout/ under core/ as pure calculation module
- **Contract:**
  - Pre: Layout in wrong layer
  - Post: Layout under core/, all functions pure
  - Inv: Same layout algorithms
- **Dependencies:** P1-006

### P2-006: Create interaction/ module for UI state machine
- **Type:** feature
- **Priority:** 1
- **Effort:** 3hr
- **Target:**
  ```
  ui/interaction/
  ├── mod.rs          (~100 lines)
  ├── state.rs        (~200 lines - InteractionMode)
  ├── transitions.rs  (~300 lines - state machine)
  └── handlers.rs     (~300 lines - event handlers)
  ```
- **Contract:**
  - Pre: Interaction logic scattered
  - Post: Explicit state machine in one module
  - Inv: Same interaction behavior
- **Dependencies:** P1-004, P1-005

### P2-007: Add module-level documentation with invariants
- **Type:** task
- **Priority:** 1
- **Effort:** 2hr
- **Target:** Every mod.rs has:
  - Module purpose (1 sentence)
  - Design by Contract section (P/Q/I)
  - Examples (if public API)
- **Contract:**
  - Pre: Inconsistent documentation
  - Post: All modules documented with contracts
  - Inv: No behavior changes
- **Dependencies:** Phase 2 refactors complete

---

## Phase 3: Documentation & Schema (Priority 2)

### P3-001: Create CUE schemas for JSON boundaries
- **Type:** feature
- **Priority:** 2
- **Effort:** 3hr
- **Target:**
  ```
  schemas/
  ├── document.cue    (DiagramDocument schema)
  ├── envelope.cue    (EventEnvelope schema)
  ├── cli_output.cue  (CLI JSON output schema)
  └── patch.cue       (JSON Patch schema)
  ```
- **Contract:**
  - Pre: JSON validated by code only
  - Post: CUE schemas validate all boundaries
  - Inv: Schemas match Rust types exactly
- **Dependencies:** P2-002 (event consolidation)

### P3-002: Document Data-Calc-Actions layer boundaries
- **Type:** task
- **Priority:** 2
- **Effort:** 1hr
- **Update:** `docs/04_DATA_CALC_ACTIONS.md`
- **Add:**
  - Explicit layer checklist
  - Anti-patterns to avoid
  - Example refactoring showing layer migration
- **Dependencies:** Phase 2 complete

### P3-003: Create architecture decision records
- **Type:** task
- **Priority:** 2
- **Effort:** 2hr
- **Target:**
  ```
  docs/adr/
  ├── 013-cupid-principles.md
  ├── 014-error-taxonomy.md
  ├── 015-module-size-limits.md
  └── 016-test-protection-policy.md
  ```
- **Dependencies:** Phase 2 complete

### P3-004: Add property-based tests for pure functions
- **Type:** task
- **Priority:** 2
- **Effort:** 3hr
- **Target:** Every pure function in core/ has proptest
- **Contract:**
  - Pre: Some pure functions lack property tests
  - Post: 100% coverage of core/ with proptest
  - Inv: Tests verify mathematical properties
- **Dependencies:** P1-006, P2-005

### P3-005: Create test fixtures for all interaction modes
- **Type:** task
- **Priority:** 2
- **Effort:** 2hr
- **Target:**
  ```
  tests/fixtures/interaction/
  ├── select_mode.json
  ├── drag_mode.json
  ├── resize_mode.json
  ├── edge_drawing.json
  └── marquee.json
  ```
- **Contract:**
  - Pre: Interaction tests use inline data
  - Post: Fixtures for all InteractionMode variants
  - Inv: Fixtures load into valid DiagramDocument
- **Dependencies:** P2-006

### P3-006: Document CUPID compliance checklist
- **Type:** task
- **Priority:** 2
- **Effort:** 1hr
- **Target:** `docs/14_CUPID_CHECKLIST.md`
- **Content:**
  - Composable: Module coupling guidelines
  - Unix-like: Single responsibility checklist
  - Predictable: Error handling patterns
  - Idiomatic: Rust style guide
  - Domain-driven: Ubiquitous language glossary
- **Dependencies:** Phase 2 complete

### P3-007: Add clippy lint configuration for CUPID
- **Type:** task
- **Priority:** 2
- **Effort:** 30min
- **Target:** `.clippy.toml` and `lib.rs` lints
- **Add:**
  - `#![deny(clippy::cognitive_complexity)]`
  - `#![warn(clippy::items_after_statements)]`
  - File size lint script in CI
- **Dependencies:** Phase 1 complete

### P3-008: Create migration guide for existing code
- **Type:** task
- **Priority:** 2
- **Effort:** 2hr
- **Target:** `docs/15_MIGRATION_GUIDE.md`
- **Content:**
  - How to identify layer violations
  - Step-by-step extraction process
  - Before/after examples from actual refactors
- **Dependencies:** P3-002

---

## Dependency Graph

```
Phase 1 (Critical - No Dependencies Between)
├── P1-001: mutation/pipeline.rs unwrap
├── P1-002: store_sqlx.rs unwrap
├── P1-003: backend.rs unwrap
├── P1-004: interaction_reducer decomposition
├── P1-005: canvas.rs decomposition → depends on P1-004
└── P1-006: DiagramDocument extraction

Phase 2 (Depends on Phase 1)
├── P2-001: store_sqlx decomposition → depends on P1-002
├── P2-002: Event consolidation → depends on Phase 1
├── P2-003: app.rs dispatch → depends on P1-005
├── P2-004: Error taxonomy → depends on Phase 1
├── P2-005: Layout move → depends on P1-006
├── P2-006: Interaction module → depends on P1-004, P1-005
└── P2-007: Module docs → depends on Phase 2 refactors

Phase 3 (Documentation)
├── P3-001: CUE schemas → depends on P2-002
├── P3-002: Layer docs → depends on Phase 2
├── P3-003: ADRs → depends on Phase 2
├── P3-004: Property tests → depends on P1-006, P2-005
├── P3-005: Test fixtures → depends on P2-006
├── P3-006: CUPID checklist → depends on Phase 2
├── P3-007: Clippy config → depends on Phase 1
└── P3-008: Migration guide → depends on P3-002
```

---

## Execution Strategy

### Parallel Work (Phase 1)
Can run simultaneously:
- P1-001, P1-002, P1-003 (unwrap elimination - different files)
- P1-004, P1-006 (different modules)

Then sequential:
- P1-005 (depends on P1-004)

### CI Gates
After each bead:
1. `moon run :clippy-source` passes
2. `moon run :test` passes
3. File count and line count verified

### Success Metrics
- Zero `unwrap()`/`expect()` in production code
- All files <300 lines (except test fixtures)
- All modules have documented contracts
- CUE schemas validate all JSON boundaries
