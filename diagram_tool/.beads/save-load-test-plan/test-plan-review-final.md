# Test Plan Review: save-load-test-plan

## VERDICT: REJECTED

---

## Executive Summary

The implementation is incomplete relative to the test plan. Critical testing artifacts
were planned but not delivered. The persistence module has 61.81% line coverage, zero
proptest invariants, zero fuzz targets, zero Kani harnesses, and `tests_import.rs`
contains ONLY `#[cfg(kani)]`-gated tests that do NOT run via `cargo test`.

---

## Tier 0 — Static Analysis

**[FAIL] Banned pattern scan**: The `#[ignore]` finding in ghost_diff is outside the
save-load bead but exists in the same crate. 36 tests are disabled.

**[PASS] Integration purity**: No `use crate::` paths found in `/tests/` directory.

**[FAIL] Density audit**: 934 tests / 487 functions = **1.92x** (target ≥5x) — **LETHAL**

**[FAIL] Ignored tests**: 36 `#[ignore]` tests in `ui/ghost_diff/state/` — **LETHAL**

---

## Tier 1 — Execution

**[PASS] Clippy**: Zero warnings

**[PASS] nextest**: 1530 tests pass, 84 skipped (skipped = different from ignored)

**[PASS] Ordering probe**: Consistent across thread counts

**[N/A] Insta**: Not present in this crate

---

## Tier 2 — Coverage

**[FAIL] Line coverage**: **61.81% overall** (target ≥90%) — **LETHAL**

**Persistence module breakdown:**
| File | Coverage |
|------|----------|
| `ui/toolbar/persistence/open.rs` | 17.12% |
| `ui/toolbar/persistence/common.rs` | 62.90% |
| `ui/toolbar/persistence/save.rs` | 74.04% |
| `ui/toolbar/persistence/tests_import.rs` | **0.00%** (only `#[cfg(kani)]` tests) |
| `ui/toolbar/persistence_compat/mod.rs` | 73.31% |

**[FAIL] Branch coverage**: 60.14% (target ≥90%)

---

## Tier 3 — Mutation

**[NOT RUN]** Mutation testing requires `--in-diff HEAD` but this review is of the
entire test suite, not a diff. Kill rate cannot be assessed.

---

## LETHAL FINDINGS

### 1. `#[ignore]` tests (36 total)
**File**: `diagram_tool/src/ui/ghost_diff/state/basic_tests.rs:13,20,31,38,49,60,76,90`
**File**: `diagram_tool/src/ui/ghost_diff/state/contract_receive_tests.rs:13,14,31,45,60,69,86`
**File**: `diagram_tool/src/ui/ghost_diff/state/contract_toggle_tests.rs:13,28,43,56,73,90,107,124,140,158`
**File**: `diagram_tool/src/ui/ghost_diff/state/contract_accept_tests.rs:13,27,43,61,73,87,101,117,135,147,160,174,188,204,218,234,252,264`

**Finding**: Tests marked `#[ignore]` are explicitly disabled. These represent
unverified behavior being shipped. Per Holzmann Rule 10 and the test-reviewer
protocol, any `#[ignore]` = **LETHAL**.

---

### 2. Overall coverage 61.81% < 90% threshold

**Finding**: The entire crate sits at 61.81% line coverage. The persistence module
is severely under-tested:
- `open.rs`: 17.12% (131 uncovered lines)
- `common.rs`: 62.90% (32 uncovered lines)
- `save.rs`: 74.04% (89 uncovered lines)
- `tests_import.rs`: 0.00% (all tests gated behind `#[cfg(kani)]`)

Per the test-reviewer protocol: Line coverage < 90% overall = **LETHAL**.

---

### 3. Density ratio 1.92x < 5x target

**Finding**: 934 tests / 487 public functions = **1.92x**

The test plan specified:
- 52 unit tests for save/load alone
- 16 integration tests
- 2 E2E tests
- 4 proptest invariants
- 3 fuzz targets
- 2 Kani harnesses

Actual:
- No proptest invariants in persistence module
- No fuzz targets (no `fuzz/` directory)
- No Kani harnesses (no `kani/` directory)
- `tests_import.rs` ONLY has `#[cfg(kani)]` tests (not run by default!)

Per the test-reviewer protocol: Ratio < 5× = **LETHAL**.

---

## MAJOR FINDINGS (5)

### 4. `tests_import.rs` contains ONLY `#[cfg(kani)]` tests

**File**: `diagram_tool/src/ui/toolbar/persistence/tests_import.rs`

**Evidence**:
```rust
#[cfg(kani)]
#[kani::proof]
fn given_malformed_import_when_preparing_transition_then_returns_parse_error() {
```

All 6 test functions in this file are gated behind `#[cfg(kani)]`. Running `cargo test`
does NOT execute these tests. The `apply_import_contents` and `prepare_import_transition`
functions have **ZERO** test coverage via normal test runs.

**Impact**: The entire import transition logic (Behaviors 34-42 in the test plan)
has no tests running. This represents a massive gap in the testing trophy.

---

### 5. Planned proptest invariants not implemented

**Test plan specified**:
- `apply_save_document_revision_sync_invariant` (Section 4.1)
- `apply_open_document_file_path_preservation_invariant` (Section 4.2)
- `apply_import_contents_atomicity_invariant` (Section 4.3)
- `serialization_roundtrip_invariant` (Section 4.4)

**Finding**: `grep -rn "proptest!" diagram_tool/src/ui/toolbar/persistence/` returns nothing.
None of the 4 planned proptest invariants exist in the persistence module.

---

### 6. Planned fuzz targets not implemented

**Test plan specified** (Section 5):
- `parse_diagram_document_with_compat` fuzz target
- `save_workspace_atomic` fuzz target
- `apply_import_contents` fuzz target

**Finding**: `ls diagram_tool/fuzz/` returns "No fuzz directory". The 3 planned fuzz
targets do not exist.

---

### 7. Planned Kani harnesses not implemented

**Test plan specified** (Section 6):
- `apply_save_document_rejects_path_traversal` (Section 6.1)
- `apply_import_contents_preserves_state_on_error` (Section 6.2)

**Finding**: `ls diagram_tool/kani/` returns "No kani directory". The 2 planned Kani
harnesses do not exist. While the test file `tests_import.rs` contains `#[kani::proof]`
functions, they are guarded by `#[cfg(kani)]` and not run via normal test.

---

### 8. Persistence open.rs severely under-tested (17.12%)

**File**: `diagram_tool/src/ui/toolbar/persistence/open.rs:146`

121 lines uncovered. The `open_workspace` action function and its WASM/native
variants have minimal test coverage. Behaviors 17-31 (open scenarios) are not
fully tested.

---

## Contract Parity Check (Mode 1 - Plan Inquisition Reprise)

### Public Functions in Contract vs Tests

| Function | Has Tests? | Test Name(s) |
|----------|------------|--------------|
| `apply_save_document` (save.rs:32) | YES | `apply_save_document_writes_file_and_clears_dirty_flag`, `apply_save_document_returns_io_error_for_invalid_path`, `apply_save_document_syncs_revision_from_saved_document`, `apply_save_document_preserves_file_path` |
| `apply_save_document` (save.rs:66, WASM) | NO | None — WASM variant not tested |
| `save_workspace` (save.rs:74) | NO | None — action function not tested |
| `apply_open_document` (open.rs:56) | YES | `apply_open_document_creates_session_with_file_path`, `apply_open_document_returns_parse_error_for_invalid_json`, `apply_open_document_returns_error_for_missing_version`, `apply_open_document_resets_revision_to_initial` |
| `open_workspace` (open.rs:296) | NO | None — action function not tested |
| `prepare_import_transition` (common.rs:14) | NO (only kani) | All tests gated `#[cfg(kani)]` |
| `apply_import_contents` (common.rs:39) | NO (only kani) | All tests gated `#[cfg(kani)]` |
| `update_load_save_success` (common.rs:55) | NO | None |
| `update_load_save_error` (common.rs:64) | NO | None |
| `use_global_keyboard` (hooks/keyboard.rs) | NO | None |

**Missing tests for public functions**:
- `save_workspace()` — 0 tests (action function)
- `open_workspace()` — 0 tests (action function)
- `prepare_import_transition()` — 0 tests via cargo test (only kani)
- `apply_import_contents()` — 0 tests via cargo test (only kani)
- `update_load_save_success()` — 0 tests
- `update_load_save_error()` — 0 tests
- `use_global_keyboard()` — 0 tests
- WASM variant of `apply_save_document` — 0 tests

---

## Assertion Sharpness Check

### Tests with weak assertions

**`open_tests.rs:48`**: `assert!(result.is_err());` — Does not assert the **exact variant**
(`OpenError::Validation` vs `OpenError::Parse` vs `OpenError::Io`). Per the test plan,
Behavior 31 specifies "returns Validation error for schema violations" but the test
only checks `is_err()` without specifying the variant.

**`tests_import.rs`**: All assertions use `matches!(result, Err(ImportTransitionError::Parse(_)))`
which IS correct for the Kani proofs, but these tests don't run via `cargo test`.

---

## Trophy Allocation (Per Test Plan)

| Layer | Planned | Actual | Gap |
|-------|---------|--------|-----|
| Unit | 52 | ~15 (persistence module) | -37 |
| Integration | 16 | 0 | -16 |
| E2E | 2 | 0 | -2 |
| Proptest | 4 | 0 | -4 |
| Fuzz | 3 | 0 | -3 |
| Kani | 2 | 0 | -2 |

---

## MANDATE

The following MUST exist before this bead can be APPROVED:

1. **Remove all 36 `#[ignore]` tests** — These are ship-stopping gaps. Either fix the
   underlying issues or delete the tests entirely.

2. **Implement the 4 planned proptest invariants** for the persistence module:
   - `apply_save_document_revision_sync_invariant`
   - `apply_open_document_file_path_preservation_invariant`
   - `apply_import_contents_atomicity_on_error_invariant`
   - `serialization_roundtrip_invariant`

3. **Implement the 3 planned fuzz targets**:
   - `parse_diagram_document_with_compat` fuzz target
   - `save_workspace_atomic` fuzz target
   - `apply_import_contents` fuzz target

4. **Implement the 2 planned Kani harnesses**:
   - `apply_save_document_rejects_path_traversal`
   - `apply_import_contents_preserves_state_on_error`

5. **Convert `tests_import.rs` to run via normal `cargo test`** — The
   `#[cfg(kani)]` guard means these 6 test functions are NOT run by default.
   Either remove the guard or add non-kani test variants.

6. **Add tests for `save_workspace()` and `open_workspace()`** — These are action
   functions with 0 tests each. They may require integration test structure.

7. **Add tests for `update_load_save_success` and `update_load_save_error`** — These
   toast helper functions have 0 tests.

8. **Increase overall coverage to ≥90%** — The persistence module needs significant
   additional test coverage, especially `open.rs` at 17.12%.

9. **Fix weak assertion in `apply_open_document_returns_error_for_missing_version`** —
   Change `assert!(result.is_err())` to `assert!(matches!(result, Err(OpenError::Validation(_))))`.

---

## Summary

| Severity | Count | Threshold | Status |
|----------|-------|-----------|--------|
| LETHAL | 3 | Any = REJECT | **REJECTED** |
| MAJOR | 5 | ≥3 = REJECT | REJECTED |
| MINOR | TBD | ≥5 = REJECT | TBD |

**LETHAL findings**: Ignored tests (36), Coverage 61.81% < 90%, Density 1.92x < 5x

**REJECTED — Full re-review required from Tier 0 after fixes.**
