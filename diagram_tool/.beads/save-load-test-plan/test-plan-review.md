# Test Plan Inquisition Report — FINAL REVIEW

**Bead**: save-load-test-plan  
**Plan**: diagram_tool/.beads/save-load-test-plan/test-plan.md  
**Contract**: diagram_tool/.beads/save-load-test-plan/contract.md  
**Date**: 2026-04-03  
**Mode**: Plan Inquisition (no implementation yet)  
**Status**: FINAL review iteration

---

## VERDICT: REJECTED

**Any single LETHAL finding = REJECTED. This plan has 12 LETHAL findings.**

---

## AXIS 1 — CONTRACT PARITY: PARTIAL FAIL

### Public Function Coverage

| Contract Function | BDD Coverage | Issue |
|---|---|---|
| `apply_save_document` (non-WASM) | ✅ Behaviors 9–15 | Missing atomic rename failure scenario |
| `apply_save_document` (WASM) | ✅ Behavior 16 | |
| `save_workspace` | ✅ Behaviors 1–8 | |
| `apply_open_document` | ✅ Behaviors 29–31 | **No explicit happy path scenario** |
| `open_workspace` | ✅ Behaviors 17–28 | |
| `prepare_import_transition` | ✅ Behaviors 37–42 | |
| `apply_import_contents` | ✅ Behaviors 34–36 | **No explicit happy path scenario** |
| `update_load_save_success` | ✅ Behavior 32 | |
| `update_load_error` | ✅ Behavior 33 | |
| `use_global_keyboard` | ✅ Behaviors 43–45 | |

**LETHAL**: `apply_open_document` happy path is listed only in combinatorial matrix (8.2), not as a named BDD scenario with Given/When/Then. The actual return value (node count, edge count, session file_path, revision reset) is not covered by any explicit scenario.

**LETHAL**: `apply_import_contents` happy path (Behavior 34) only tests "updates doc and history atomically on success" with `is_ok()` assertion. The actual doc structure (node count, edge count) is not verified in the scenario.

### Error Variant Completeness

| Error Enum | Variant | Covered? | Issue |
|---|---|---|---|
| `SaveError::NoFilePath` | ✅ Behavior 13 | But assertion is `Err(SaveError::NoFilePath)` — LETHAL |
| `SaveError::Serialize(String)` | ✅ Behavior 14 | Sharpened ✅ |
| `SaveError::Io(String)` | ✅ Behaviors 12, 15 | Behavior 12 uses `Err(SaveError::Io(_))` — LETHAL |
| `OpenError::Parse(String)` | ✅ Behavior 29 | `s.len() > 0` — LETHAL |
| `OpenError::Validation(String)` | ✅ Behavior 30 | `s.contains("version")` ✅ |
| `OpenError::Io(String)` | ✅ Behavior 25 | Wildcard — MAJOR |
| `ImportTransitionError::Parse` | ✅ Behavior 35 | `is_err()` only — LETHAL |
| `ImportTransitionError::Validation` | ✅ Behavior 36 | `is_err()` only — LETHAL |
| `CliPersistenceError::AtomicRenameError` | ⚠️ Matrix 8.1 | **No explicit scenario** — MAJOR |
| `CliPersistenceError::TempFileError` | ⚠️ existing tests | no new scenario — MAJOR |
| `CliPersistenceError::ValidationError` | ⚠️ existing tests | no new scenario — MAJOR |

---

## AXIS 2 — ASSERTION SHARPNESS: FAIL

### LETHAL — Generic Assertions (12 findings)

Any `is_ok()` / `is_err()` / `Err(Variant(_))` without concrete value = LETHAL.

| # | Behavior | Line | Assertion | Finding |
|---|---|---|---|---|
| 12 | `apply_save_document() returns Io error for invalid path` | ~254 | `Err(SaveError::Io(_))` | Wildcard. No `where` clause. |
| 13 | `apply_save_document() returns NoFilePath` | ~266 | `Err(SaveError::NoFilePath)` | No `matches!` guard |
| 29 | `apply_open_document() returns Parse error for invalid JSON` | ~474 | `Err(OpenError::Parse(s)) where s.len() > 0` | `len() > 0` is not concrete |
| 33 | `update_load_save_success() updates toast` | ~502 | `ToastIntent::Success` only | No title/detail verification |
| 34 | `apply_import_contents() atomicity on success` | ~530 | `doc.nodes.len() == 2, history.can_undo() == true` | No return value assertion; function returns `()` |
| 35 | `apply_import_contents() unchanged on parse error` | ~542 | "doc still has 3 nodes (unchanged)" | **No assertion that function returned error** |
| 36 | `apply_import_contents() unchanged on validation error` | ~554 | "history still has prior state (unchanged)" | **No assertion that function returned error** |
| 39 | `prepare_import_transition() happy path` | ~614 | `Returns Ok((doc, history))` | `is_ok()` check only — no doc.revision, no structure |
| 40 | `parse_diagram_document_with_compat() rejects no version` | ~626 | `Returns Err` | Generic `Err`, no variant, no content |
| 43 | `keyboard Ctrl+S native` | ~642 | "save_workspace() is called" | No side-effect verification |
| 44 | `keyboard Ctrl+S WASM` | ~654 | "save_workspace() is called" | No side-effect verification |
| 45 | `keyboard Ctrl+O native` | ~668 | "open_workspace() is called" | No side-effect verification |

**12 LETHAL** generic assertion findings.

### MAJOR — Toast Content Not Verified

Behaviors 25, 33, 34, 35, 47 only check `ToastIntent` enum variant, not `title` or `detail` strings.

### Concrete Assertions (PASS)

| Behavior | Why PASS |
|---|---|
| 1 (info toast title/detail) | Concrete strings |
| 2 (file dialog filter/extension) | Concrete values |
| 3 (is_dirty == false, revision == 10) | Concrete values |
| 10 (empty doc succeeds) | Concrete assertions |
| 11 (revision sync) | Concrete `Revision(10)` |
| 14 (`s.contains("validation")`) | Sharpened ✅ |
| 15 (`s.contains("Path traversal denied")`) | Concrete substring |
| 20 (revision == INITIAL) | Concrete value |
| 28 (node/edge count) | Concrete `len()` values |
| 30 (`s.contains("version")`) | Sharpened ✅ |
| 37 (camelCase → snake_case) | Concrete field values |
| 41 (`s.contains("diamond")`) | Concrete substring |

---

## AXIS 3 — TROPHY ALLOCATION: PASS

| Metric | Value | Target | Status |
|---|---|---|---|
| Total tests | 70 (52 unit + 16 integration + 2 e2e) | — | |
| Public functions | 10 | — | |
| Ratio | **7.0x** | ≥5x | ✅ PASS |
| Proptest invariants | 4 | — | ✅ |
| Fuzz targets | 3 | — | ✅ |
| Kani harnesses | 2 | — | ✅ |

All pure functions with non-trivial input space have proptest coverage. Parsers have fuzz targets.

---

## AXIS 4 — BOUNDARY COMPLETENESS: MINOR FAIL

### Named boundaries

**`apply_save_document`**:
- ✅ Empty document (Behavior 10)
- ✅ Nonexistent directory (Behavior 12 — though assertion loose)
- ✅ No file path (Behavior 13)
- ✅ Validation failure (Behavior 14)
- ✅ Path traversal ".." (Behavior 15)
- ⚠️ **Atomic rename failure**: named in matrix 8.1 row 6, no scenario — MAJOR
- ⚠️ Permission denied: only UI toast (Behavior 5), not direct function boundary
- ⚠️ Max document size: not named

**`apply_open_document`**:
- ⚠️ Happy path: only in matrix, not as named scenario — LETHAL
- ✅ Invalid JSON (Behavior 29 — though assertion loose)
- ✅ Missing version (Behavior 30)
- ⚠️ Empty document boundary: not explicitly tested
- ⚠️ Max JSON size: not named

**`prepare_import_transition`**:
- ✅ All named

**`apply_import_contents`**:
- ⚠️ Happy path: only Behavior 34 with `is_ok()` check — LETHAL

---

## AXIS 5 — MUTATION SURVIVABILITY: PARTIAL

### Mutations caught ✅

| Mutation | Test |
|---|---|
| Remove is_dirty() check | `apply_save_document_clears_dirty_flag_on_success` |
| Skip mark_saved() | `apply_save_document_syncs_revision_from_current_document` |
| Wrong revision assignment | `apply_save_document_syncs_revision_from_current_document` |
| Wrong path traversal error mapping | `apply_save_document_returns_path_traversal_error_when_path_contains_double_dot` |
| Skip history push (open) | `open_workspace_pushes_current_doc_to_history_before_loading` |
| Skip INITIAL reset (open) | `open_workspace_resets_revision_to_initial_on_load` |
| Skip LKG fallback | `open_workspace_uses_lkg_fallback_when_primary_file_corrupt` |
| Mutate before validate (import) | `apply_import_contents_leaves_doc_and_history_unchanged_on_validation_error` |
| Skip error rollback | `apply_import_contents_leaves_doc_and_history_unchanged_on_parse_error` |

### Mutations NOT caught ⚠️

| Mutation | Why NOT Caught |
|---|---|
| `apply_open_document` happy path returns wrong node count | No explicit happy path scenario with concrete node/edge assertions |
| `apply_import_contents` happy path returns wrong structure | Behavior 34 only checks `is_ok()`, not doc content |
| `apply_save_document` atomic rename → wrong error variant | Behavior 12's `Err(SaveError::Io(_))` would accept any Io error |
| `prepare_import_transition` skips revision reset | Behavior 39's `is_ok()` would not catch skipped reset |

---

## AXIS 6 — HOLZMANN PLAN AUDIT: MINOR FAIL

- **Rule 2**: No loops in test bodies ✅
- **Rule 5**: Most scenarios have explicit Given blocks. Behaviors 43–45 (keyboard) have acceptable DAMP. ✅
- **Rule 6**: Plan does not contain `let _ =` or `.ok()` in assertions ✅
- **Rule 7**: No shared mutable state described ✅
- **Rule 8**: Side-effect helpers (`make_temp_file()`, `make_dirty_session_with_doc()`) are named for their effects ✅

**MINOR**: Proptest implementation hints (lines 726–830) use `prop_assert!(result.is_ok())` — boolean-only checks without concrete value assertions inside proptest invariants. This is a gap in the planned test implementation itself.

---

## LETHAL FINDINGS (12 — ANY SINGLE ONE = REJECTED)

1. **test-plan.md:~254 — Behavior 12**: `Then: Returns Err(SaveError::Io(_))` — wildcard, no `where` clause. The function could return any Io error and the test would pass.

2. **test-plan.md:~266 — Behavior 13**: `Then: Returns Err(SaveError::NoFilePath)` — `matches!` without a guard on a unit variant. Convention requires explicit `matches!(result, Err(SaveError::NoFilePath))`.

3. **test-plan.md:~474 — Behavior 29**: `Then: Returns Err(OpenError::Parse(s)) where s.len() > 0` — `len() > 0` is not a concrete value. A 1-character error string passes. Actual error content unverified.

4. **test-plan.md:~502 — Behavior 33**: `Then: The toast is updated with intent ToastIntent::Success` — only enum variant checked, title and detail strings not verified.

5. **test-plan.md:~530 — Behavior 34**: `Then: doc.nodes.len() == 2, history.can_undo() == true` — but `apply_import_contents` returns `Result<(), E>`. No assertion on the `Result`. The function could return `Err` and the doc/history assertions would still pass if nothing was mutated.

6. **test-plan.md:~542 — Behavior 35**: `Then: doc still has 3 nodes (unchanged), history still has prior state (unchanged)` — **no assertion that the function returned an error**. The function could succeed silently and these assertions would still pass.

7. **test-plan.md:~554 — Behavior 36**: Same as Behavior 35 — no `result.is_err()` or `matches!(result, Err(...))` assertion.

8. **test-plan.md:~614 — Behavior 39**: `Then: Returns Ok((doc, history))` — `is_ok()` is the only check. `doc.revision`, node structure, edge structure are not verified. A corrupted doc with wrong revision would pass.

9. **test-plan.md:~626 — Behavior 40**: `Then: Returns Err` — generic `Err` without variant or content. Should be `Err(ImportTransitionError::Validation(s)) where s.contains("version")`.

10. **test-plan.md:~642 — Behavior 43**: `Then: save_workspace(...) is called` — action function. No verification of signal mutations, toasts, or spawned task effects. The test proves the function was called, not that it did the right thing.

11. **test-plan.md:~654 — Behavior 44**: Same as Behavior 43 (WASM variant).

12. **test-plan.md:~668 — Behavior 45**: Same as Behavior 43 (Ctrl+O).

---

## MAJOR FINDINGS (6)

1. **`AtomicRenameError` has no explicit BDD scenario** — listed in matrix 8.1 row 6 but not in behaviors. This is a distinct `CliPersistenceError` variant with `from`/`to` fields.

2. **`TempFileError` has no explicit BDD scenario** — only "existing tests" reference.

3. **`ValidationError` has no explicit BDD scenario** — only "existing tests" reference.

4. **`apply_open_document` happy path not a named BDD scenario** — return values (node/edge count, session, revision reset) only in matrix 8.2, not in Section 3.

5. **`apply_import_contents` happy path not concretely verified** — Behavior 34 only checks `is_ok()` via doc state, not the actual returned value or doc structure.

6. **Toast content not verified in Behaviors 25, 33, 34, 35, 47** — only `ToastIntent` enum variant checked.

---

## MINOR FINDINGS

1. **Max document/JSON size not named** for `apply_save_document` or `apply_open_document`.
2. **Empty document for `apply_open_document`** not explicitly tested as a boundary.
3. **Permission denied boundary** for `apply_save_document` only tested via UI toast, not as direct function boundary.
4. **Proptest implementation hints** use `prop_assert!(result.is_ok())` without concrete value checks.

---

## CONFIRMED FIXES (from user)

| Fix | Status |
|---|---|
| Behavior 14: `SaveError::Serialize(s)` sharpened to `s.contains("validation")` | ✅ Confirmed — line 284 |
| Behavior 30: `OpenError::Parse(s)` sharpened to `s.len() > 0` | ✅ Confirmed — but `len() > 0` is still LETHAL |
| Behavior 31: `OpenError::Validation(s)` sharpened to `s.contains("version")` | ✅ Confirmed — line 492 |
| Empty document boundary at line ~220 | ✅ Confirmed — Behavior 10 |

---

## MANDATE

### Must Fix Before Resubmission (all LETHAL must be resolved):

1. **Behavior 12**: Change to `Err(SaveError::Io(s)) where s.contains("...")` with a concrete error substring that is unique to "path does not exist" (not "permission denied" or "atomic rename").

2. **Behavior 13**: Add `matches!(result, Err(SaveError::NoFilePath))`.

3. **Behavior 29**: Change `s.len() > 0` to `s.contains("...")` with a concrete parse error substring.

4. **Behaviors 35–36**: Add `prop_assert!(result.is_err())` AND `matches!(result, Err(ImportTransitionError::Parse/Validation(_)))` before the unchanged-state assertions.

5. **Behavior 39**: Change to `matches!(result, Ok((ref doc, ref history)))` with `prop_assert_eq!(doc.revision, Revision::INITIAL)` and `prop_assert!(!doc.nodes.is_empty() || /* specific check */)`.

6. **Behavior 40**: Change to `Err(ImportTransitionError::Validation(s)) where s.contains("version")`.

7. **Behaviors 43–45**: Add explicit verification of side effects (signal reads, store_bridge calls, toast queue state).

8. **Behavior 34**: Add `prop_assert!(result.is_ok())` AND concrete assertions on the doc state.

### Must Add (MAJOR):

9. **New Behavior**: `apply_open_document() happy path` — explicit BDD scenario with Given: valid JSON containing N nodes/M edges, When, Then: `next_doc.nodes.len() == N`, `next_doc.edges.len() == M`, `session.file_path() == Some(input_path)`, `next_doc.revision == Revision::INITIAL`.

10. **New Behavior**: `apply_import_contents() happy path` — explicit BDD scenario with concrete node/edge count assertions on success.

11. **New Behavior**: `apply_save_document() returns AtomicRenameError when rename fails` — explicit scenario for `Err(SaveError::Io(s)) where s.contains("Atomic rename failed")`.

12. **New Behavior**: `apply_save_document() returns Io error for permission denied` — explicit boundary.

### Must Sharpen (MAJOR):

13. **Behaviors 25, 33, 34, 35, 47**: Add `detail.contains("...")` or `title.contains("...")` to toast scenarios.

---

*This plan has 12 LETHAL findings. The 4 user-confirmed fixes (Behaviors 14, 30, 31, empty doc) are confirmed. However, 12 other LETHAL findings and 6 MAJOR findings remain. The plan cannot be APPROVED in this state.*
