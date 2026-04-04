# Test Plan Review: File Save/Load Persistence Feature

**Plan**: `diagram_tool/.beads/save-load-test-plan/test-plan.md`
**Contract**: `diagram_tool/.beads/save-load-test-plan/contract.md`
**Mode**: Plan Inquisition (no implementation yet)
**Date**: 2026-04-04

---

## VERDICT: REJECTED

---

## Axis 1 — Contract Parity: PASS (with caveat)

| Function | Contract Location | Covered By Behaviors |
|----------|-----------------|---------------------|
| `apply_save_document` (non-WASM) | save.rs:32 | Behaviors 9-16 |
| `apply_save_document` (WASM) | save.rs:54 | Behavior 16 |
| `save_workspace` | save.rs:62 | Behaviors 1-8 |
| `apply_open_document` | open.rs:56 | Behaviors 28-31 |
| `open_workspace` | open.rs:296 | Behaviors 17-27 |
| `prepare_import_transition` | common.rs:14 | Behaviors 37-42 |
| `apply_import_contents` | common.rs:39 | Behaviors 34-36 |
| `update_load_save_success` | common.rs:55 | Behavior 32 |
| `update_load_save_error` | common.rs:64 | Behavior 33 |
| `use_global_keyboard` | keyboard.rs:21,181 | Behaviors 43-45 |

**All 10 functions have at least one BDD scenario.**

**Caveat**: `use_global_keyboard` WASM variant (keyboard.rs:181) - the plan references behaviors 43-45 but Behavior 45 (Ctrl+O on native only) is non-WASM only. WASM keyboard coverage is only Behavior 44. The contract shows different signatures for WASM vs native but the plan does not explicitly test both paths for `use_global_keyboard`.

---

## Axis 2 — Assertion Sharpness: FAIL (LETHAL)

### LETHAL Finding: Wildcard assertion on SaveError::Serialize

**Behavior 14** (`apply_save_document_returns_serialize_error_when_document_validation_fails`):

```
Then: Returns `Err(SaveError::Serialize(s))` where `s.contains("validation") || s.contains("schema")`
```

**Problem**: The error message string is implementation-defined. Any string containing "validation" OR "schema" anywhere passes. This means:
- `Err(SaveError::Serialize("schema validation failed".to_string()))` ✓
- `Err(SaveError::Serialize("validation: something".to_string()))` ✓
- `Err(SaveError::Serialize("The schema...".to_string()))` ✓
- `Err(SaveError::Serialize("validation".to_string()))` ✓

But also:
- `Err(SaveError::Serialize("validation_error".to_string()))` ✓
- `Err(SaveError::Serialize("schema!".to_string()))` ✓ (no "validation" present!)

This is a wildcard. The plan itself shows the problem: "serialize error when document validation fails" — if the validation error message doesn't contain "validation" OR "schema", the test fails even though it's the correct error variant.

**Required fix**: The assertion must be `Err(SaveError::Serialize(_))` with NO string check, OR the string check must be removed from the behavior description entirely. The Error variant alone is what matters, not the inner string content.

### MAJOR Finding: OpenError::Validation has no scenario

The error coverage matrix (Section 9) lists:
```
| Validation(String) | (other schema violations) | apply_open_document_returns_validation_error_for_schema_violations |
```

But `apply_open_document_returns_validation_error_for_schema_violations` does NOT appear as a scenario in Section 3.2. Behaviors 28-31 are listed but only Behaviors 29 and 30 have explicit scenarios. Behavior 31 (Validation error) is referenced by name but no scenario body exists.

**Required fix**: Add explicit scenario:
```
#### Behavior: apply_open_document() returns Validation error for schema violations

**Given**: Valid current document, JSON that is valid JSON but fails diagram schema validation

**When**: `apply_open_document(&current_doc, &history, invalid_schema_json, path)` is called

**Then**: Returns `Err(OpenError::Validation(s))` where `s.len() > 0`
```

### MAJOR Finding: CliPersistenceError variant coverage is unverified

Section 9 lists:
- `TempFileError` → "cli_persistence/tests.rs" (existing)
- `AtomicRenameError` → "cli_persistence/tests.rs" (existing)

But:
1. No test name is given
2. No scenario in this plan references these variants with exact assertions
3. The plan cannot verify that external tests cover these exact variants

**Required fix**: Either add explicit scenarios in this plan for these error variants, or explicitly name the exact test functions in `cli_persistence/tests.rs` that cover each variant.

---

## Axis 3 — Trophy Allocation: PASS

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| Unit tests | 52 | ≥50 (5×10 functions) | ✓ PASS |
| Integration tests | 16 | Reasonable for I/O layer | ✓ PASS |
| E2E tests | 2 | Minimal for keyboard/dialogs | ✓ PASS |
| Static | 0 | Covered by clippy | ✓ PASS |
| Proptest invariants | 4 | Adequate for pure functions | ✓ PASS |
| Fuzz targets | 3 | Adequate for parsers | ✓ PASS |
| Kani harnesses | 2 | Adequate for critical invariants | ✓ PASS |

**Ratio**: 52/10 = 5.2× — meets the ≥5× threshold.

---

## Axis 4 — Boundary Completeness: FAIL (MAJOR)

### apply_save_document boundaries — MISSING:

| Boundary | Named? |
|----------|--------|
| Minimum valid input (empty doc) | ✓ Behavior 10 |
| Maximum valid input | ✗ MISSING |
| One-above-maximum (too large doc) | ✗ MISSING |
| Empty / zero | ✗ MISSING (empty doc ≠ empty file path) |
| Overflow potential | ✗ MISSING (revision u64 overflow) |

**Missing test**: What happens when document has u64::MAX revisions? What happens with maximum node/edge count?

### prepare_import_transition boundaries — MISSING:

| Boundary | Named? |
|----------|--------|
| Empty string input | ✗ MISSING |
| Maximum JSON size | ✗ MISSING (1MB limit?) |
| Deeply nested JSON | ✗ MISSING |
| JSON with extremely long strings | ✗ MISSING |

**Missing test**: What happens with empty string `""`? What happens with 100MB JSON?

### apply_import_contents boundaries — MISSING:

| Boundary | Named? |
|----------|--------|
| Empty string input | ✗ MISSING |
| Valid JSON but wrong types | ✗ MISSING |
| JSON with wrong structure types | ✗ MISSING |

**Note**: Fuzz target 5.3 covers `apply_import_contents` but the plan does not explicitly name these as boundary cases.

**Aggregation**: ≥3 missing boundaries on one function = MAJOR. Several functions have ≥3 missing boundaries.

---

## Axis 5 — Mutation Survivability: PARTIAL

### Section 7 Mutation Checkpoints — Analysis:

| Mutation | Claimed Test | Reality |
|----------|--------------|---------|
| Remove `is_dirty()` check | `apply_save_document_clears_dirty_flag_on_success` | ✓ Test exists |
| Skip `mark_saved()` call | `apply_save_document_syncs_revision_from_current_document` | ✓ Test exists |
| Skip `fsync()` | `save_workspace_atomic_persists_data_to_disk` (existing) | ✗ NOT IN THIS PLAN |
| Skip temp file cleanup | `given_atomic_save_when_complete_then_no_temp_files_remain` (existing) | ✗ NOT IN THIS PLAN |
| Wrong revision assignment | `apply_save_document_syncs_revision_from_current_document` | ✓ Test exists |
| Map wrong variant for PathTraversalDenied | `apply_save_document_returns_path_traversal_error...` | ✓ Test exists |
| Skip history push (open) | `open_workspace_pushes_current_doc_to_history_before_loading` | ✓ Test exists |
| Skip Revision::INITIAL reset | `open_workspace_resets_revision_to_initial_on_load` | ✓ Test exists |
| Skip LKG fallback | `open_workspace_uses_lkg_fallback_when_primary_file_corrupt` | ✓ Test exists |
| Skip store_bridge reset | `open_workspace_resets_store_bridge_on_native_after_load` | ✓ Test exists |
| Wrong error mapping for IO | `open_workspace_shows_error_toast_on_native_io_failure` | ✓ Test exists |
| Mutate doc BEFORE validate | `apply_import_contents_leaves_doc_and_history_unchanged_on_validation_error` | ✓ Test exists |
| Skip history push on success | `apply_import_contents_updates_doc_and_history_atomically` | ✓ Test exists |
| Skip error rollback | `apply_import_contents_leaves_doc_and_history_unchanged_on_parse_error` | ✓ Test exists |
| Skip revision reset to INITIAL | `prepare_import_transition_parses_valid_v2_json...` | ✓ Test exists |
| Skip legacy field migration | `prepare_import_transition_migrates_legacy_camelcase...` | ✓ Test exists |
| Overwrite existing icon_url | `prepare_import_transition_preserves_existing_icon_url...` | ✓ Test exists |

**Problem**: Several mutations reference "existing" tests in `cli_persistence/tests.rs` that are NOT part of this plan's scope. The plan cannot verify these tests exist or would catch the mutations.

**Mutations NOT verifiable from this plan**:
- `save_workspace_atomic_persists_data_to_disk` — external
- `given_atomic_save_when_complete_then_no_temp_files_remain` — external
- `CliPersistenceError::AtomicRenameError` coverage — external
- `CliPersistenceError::TempFileError` coverage — external

---

## Axis 6 — Holzmann Plan Audit

### Rule 2 — Bound Every Loop
Plan describes behaviors but test bodies don't exist yet. No violations detectable in plan doc.

### Rule 5 — State Your Assumptions
**MIXED**: Most scenarios have explicit `Given:` blocks. Some are vague:

**Vague** (Behavior 15):
```
**Given**: Session with dirty document, valid file path that contains ".."
```
What is a "valid file path that contains '..'"? A path like `/foo/../bar.json` or `../foo.json`? These have different semantic meanings (one is a path traversal attempt, one might be legitimate depending on context).

**Explicit** (Behavior 12):
```
**Given**: Session with dirty document

**When**: `apply_save_document(&doc, &session, &PathBuf::from("/nonexistent/dir/file.json"))`
```
This is specific: nonexistent directory.

### Rule 8 — Surface Your Side Effects
N/A for plan review (no implementation).

---

## Summary of Findings

| Severity | Count | Blocking? |
|----------|-------|-----------|
| LETHAL | 1 | YES |
| MAJOR | 3 | YES |
| MINOR | 4 | No |

### LETHAL FINDINGS

1. **`test-plan.md:284` — Wildcard assertion on SaveError::Serialize inner string**
   - `s.contains("validation") || s.contains("schema")` is not a sharp assertion
   - The Error variant `Err(SaveError::Serialize(_))` is sufficient; the string check is implementation detail
   - **Required fix**: Remove string check, assert `Err(SaveError::Serialize(_))` only

### MAJOR FINDINGS (3)

1. **`test-plan.md:1143` — OpenError::Validation has no scenario body**
   - `apply_open_document_returns_validation_error_for_schema_violations` is named but no scenario exists
   - **Required fix**: Add explicit BDD scenario with `Err(OpenError::Validation(_))` assertion

2. **`test-plan.md:1157-1161` — CliPersistenceError::TempFileError and AtomicRenameError have no test names**
   - References "existing" tests without naming them
   - Cannot verify coverage from this plan alone
   - **Required fix**: Name exact test functions or add scenarios in this plan

3. **`test-plan.md` — Boundary completeness failures on multiple functions**
   - `apply_save_document`: max doc size, overflow potential not named
   - `prepare_import_transition`: empty string, max JSON size, deeply nested JSON not named
   - `apply_import_contents`: empty string, wrong types not named
   - **Required fix**: Add explicit boundary tests or acknowledge in "Open Questions"

---

## MANDATE

Before resubmission, the following MUST be addressed:

1. **Remove wildcard assertion** (test-plan.md:284): Change `Err(SaveError::Serialize(s)) where s.contains(...)` to `Err(SaveError::Serialize(_))`

2. **Add missing scenario**: `apply_open_document_returns_validation_error_for_schema_violations` must have a scenario body in Section 3.2 with exact `Err(OpenError::Validation(_))` assertion

3. **Name external test coverage**: For `TempFileError` and `AtomicRenameError` variants, either name the exact test function in `cli_persistence/tests.rs` or add explicit scenarios in this plan

4. **Acknowledge boundary gaps**: Either add boundary tests or add to "Open Questions" section (item 2: "Is there a maximum document size?")

---

## WHAT PASSES

- All 10 public functions have at least one scenario ✓
- Trophy allocation is appropriate (5.2× ratio) ✓
- Error variant coverage matrix is mostly complete ✓
- Proptest invariants are well-designed ✓
- Fuzz targets are appropriate for parsers ✓
- Kani harnesses cover critical invariants ✓
- Mutation checkpoints are well-reasoned ✓

---

*Review completed by test-inquisitor. Resubmit with LETHAL and MAJOR findings addressed for re-review.*
