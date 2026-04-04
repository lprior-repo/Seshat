# Test Plan Review — save-load-test-plan

**Date**: 2026-04-04
**Mode**: Plan Inquisition (contract.md + test-plan.md)
**Status**: **REJECTED**

---

## Summary

The test plan has 46 behaviors across 10 public functions with good coverage ratios (5.2x unit test density). Four fixes were applied prior to this review:
1. ✓ Removed string check from Behavior 14 (wildcard `Serialize(_)` used)
2. ✓ Added BDD scenario body for schema violations validation error
3. ✓ Named test functions for TempFileError and AtomicRenameError
4. ✓ Added Open Questions section for known limitations

However, **2 MAJOR gaps remain**: `TempFileError` and `AtomicRenameError` have named test functions referenced in the plan, but **no BDD scenario body** describes what triggers these errors or what the expected behavior is. This is a coverage gap.

---

## Axis 1 — Contract Parity

| Function | Behaviors | Status |
|----------|-----------|--------|
| `apply_save_document` (native) | Behaviors 9-16 + 6 matrix rows | ✓ |
| `apply_save_document` (WASM) | Behavior 16 | ✓ |
| `save_workspace` | Behaviors 1-8 | ✓ |
| `apply_open_document` | Behaviors 28-31 | ✓ |
| `open_workspace` | Behaviors 17-27 | ✓ |
| `prepare_import_transition` | Behaviors 37-42 | ✓ |
| `apply_import_contents` | Behaviors 34-36 | ✓ |
| `update_load_save_success` | Behavior 32 | ✓ |
| `update_load_save_error` | Behavior 33 | ✓ |
| `use_global_keyboard` | Behaviors 43-45 | ✓ |

**10/10 functions have BDD scenarios.**

### Error Enum Coverage

| Enum | Variant | Covered By | Status |
|------|---------|------------|--------|
| SaveError | NoFilePath | Behavior 13 | ✓ |
| SaveError | Serialize | Behavior 14 | ✓ |
| SaveError | Io | Behaviors 12, 15 | ✓ |
| OpenError | Parse | Behaviors 30, 31 | ✓ |
| OpenError | Validation | Behavior 31 | ✓ |
| OpenError | Io | Behavior 25 (toast only) | ⚠️ See MAJOR-2 |
| ImportTransitionError | Parse | Behavior 35 | ✓ |
| ImportTransitionError | Validation | Behavior 36 | ✓ |
| CliPersistenceError | IoError | cli_persistence/tests.rs | ✓ |
| CliPersistenceError | ParseError | cli_persistence/tests.rs | ✓ |
| CliPersistenceError | ValidationError | cli_persistence/tests.rs | ✓ |
| CliPersistenceError | **TempFileError** | Named function (section 9) | ❌ **MAJOR-1** |
| CliPersistenceError | **AtomicRenameError** | Named function (section 9) | ❌ **MAJOR-2** |
| CliPersistenceError | NoValidDocument | Behaviors 26, integration | ✓ |
| CliPersistenceError | PathTraversalDenied | Behavior 15 | ✓ |

---

## Axis 2 — Assertion Sharpness

| Behavior | Then Clause | Assessment |
|----------|-------------|------------|
| 12 | `Err(SaveError::Io(_))` | ✓ Wildcard acceptable after fix |
| 13 | `Err(SaveError::NoFilePath)` | ✓ Exact variant |
| 14 | `Err(SaveError::Serialize(_))` | ✓ Wildcard acceptable after fix |
| 15 | `Err(SaveError::Io(s)) where s.contains("Path traversal denied")` | ✓ Specific string |
| 31 | `Err(OpenError::Validation(_))` | ✓ Wildcard acceptable after fix |

**No `is_ok()` or `is_err()` as assertions.** ✓

---

## Axis 3 — Trophy Allocation

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| Unit tests | 52 | ≥50 (5×10 functions) | ✓ 5.2x |
| Integration | 16 | 16 | ✓ |
| E2E | 2 | 2 | ✓ |
| Static | 0 | 0 | ✓ |
| Proptest invariants | 4 | ≥4 | ✓ |
| Fuzz targets | 3 | ≥3 | ✓ |
| Kani harnesses | 2 | ≥2 | ✓ |
| Mutation kill rate | ≥90% | ≥90% | ✓ Planned |

---

## Axis 4 — Boundary Completeness

| Function | Boundaries Named | Missing |
|----------|------------------|---------|
| `apply_save_document` | Min path, invalid path, path traversal, no session path, WASM | Atomic rename failure, temp file error |
| `apply_open_document` | Invalid JSON, missing version, schema violations | Io error (file doesn't exist), max size |
| `apply_import_contents` | Valid JSON, parse error, validation error | Empty string, max payload |
| `prepare_import_transition` | Legacy fields, icon migration, arrow types | — |

**Missing boundaries:**
- `apply_import_contents`: Empty string `""` is a distinct boundary from parse error
- Max document size: Open Question 6 flagged, no explicit boundary test planned

---

## Axis 5 — Mutation Survivability

### Missing Kill Tests

**CliPersistenceError wrapping in apply_save_document:**

The `save_workspace_atomic` function (cli_persistence layer) can return:
- `Err(CliPersistenceError::TempFileError(String))`
- `Err(CliPersistenceError::AtomicRenameError { from, to })`

The `apply_save_document` function maps these to:
- `Err(SaveError::Io(_))` — same wrapper for both

**If someone changes the mapping** (e.g., `TempFileError` → `SaveError::Serialize`), **no test in the plan would catch it** because:
1. No scenario explicitly triggers the temp file failure path
2. No scenario asserts the exact error message content for these cases
3. The "mutation checkpoints" section (7.1) does not list this as a critical mutation to catch

**Required scenarios not present:**
1. Scenario: "apply_save_document() returns Io error when atomic write temp file creation fails"
2. Scenario: "apply_save_document() returns Io error when atomic rename fails"
3. Scenario: "apply_save_document() preserves original path when atomic rename fails" (file_path preservation on error)

---

## Axis 6 — Holzmann Plan Audit

| Rule | Assessment |
|------|------------|
| Rule 1 (Linear) | All BDD scenarios have linear Given→When→Then ✓ |
| Rule 2 (Bound loops) | No loops in plan text ✓ |
| Rule 3 (Resource cleanup) | N/A for plan (would be enforced in implementation) |
| Rule 4 (One job) | Each scenario tests one behavior ✓ |
| Rule 5 (Assumptions explicit) | Given clauses explicitly state preconditions ✓ |
| Rule 6 (No swallowed errors) | Assertions use exact variants or wildcards, not bare `is_ok()`/`is_err()` ✓ |
| Rule 7 (Narrow state) | N/A for plan |
| Rule 8 (Side effects surfaced) | Test names describe the action, setup helpers not needed ✓ |
| Rule 9 (One layer magic) | All scenarios self-contained ✓ |
| Rule 10 (Warnings) | N/A for plan |

---

## VERDICT: **REJECTED**

### Severity Tally

| Severity | Count | Threshold | Action |
|----------|-------|-----------|--------|
| LETHAL | 0 | Any | — |
| MAJOR | 2 | ≥3 | Continue |
| MINOR | 3 | ≥5 | Continue |

**Status**: 0 LETHAL + 2 MAJOR + 3 MINOR → Continue to full review

---

## LETHAL FINDINGS

None.

---

## MAJOR FINDINGS (2)

### MAJOR-1: `TempFileError` has no BDD scenario body

**Location**: Section 9 (Error Enum Coverage), CliPersistenceError table

**Finding**: The plan references `TempFileError` as being covered by `cli_persistence/tests.rs` with test name `given_valid_document_when_saved_atomically_then_file_exists`. However:
1. This is an integration test for `save_workspace_atomic` directly, NOT a scenario for `apply_save_document`
2. No BDD scenario describes what input causes `TempFileError` to propagate through `apply_save_document`
3. No BDD scenario asserts that `apply_save_document` wraps `TempFileError` into `SaveError::Io` with appropriate message

**Evidence**: Section 8.1 (Combinatorial Coverage Matrix) row "Atomic rename failure" says `Err(SaveError::Io(contains "Atomic rename failed"))` but this is a **row label**, not a link to a scenario. No behavior number or scenario name is provided for this row.

**Required fix**: Add BDD scenario:
```
#### Behavior: apply_save_document() returns Io error when atomic write temp file creation fails

**Given**: Valid document and session with valid file path

**When**: `apply_save_document(&doc, &session, &path)` is called and temp file creation fails

**Then**: Returns `Err(SaveError::Io(s))` where `s` indicates temp file creation failure

**Test name**: `fn apply_save_document_returns_io_error_when_temp_file_creation_fails()`
```

### MAJOR-2: `AtomicRenameError` has no BDD scenario body

**Location**: Same as MAJOR-1

**Finding**: Same analysis as MAJOR-1 but for `AtomicRenameError`.

**Evidence**: The combinatorial matrix (section 8.1) row "Atomic rename failure" is an orphaned row — no behavior number, no scenario link, no test name that describes this specific error path.

**Required fix**: Add BDD scenario:
```
#### Behavior: apply_save_document() returns Io error when atomic rename fails

**Given**: Valid document and session with valid file path

**When**: `apply_save_document(&doc, &session, &path)` is called and the final rename operation fails

**Then**: Returns `Err(SaveError::Io(s))` where `s` indicates atomic rename failure

**Test name**: `fn apply_save_document_returns_io_error_when_atomic_rename_fails()`
```

---

## MINOR FINDINGS (3)

### MINOR-1: `apply_import_contents` empty string boundary not named

**Location**: Section 4 (Boundary Completeness)

**Finding**: The empty string `""` is a distinct boundary from parse error (which requires non-empty but malformed JSON). No scenario explicitly tests the `""` input.

**Required fix**: Add to boundary list or add explicit scenario.

### MINOR-2: `apply_open_document` Io error not tested at calc layer

**Location**: Behavior 25 tests the toast, but `apply_open_document` itself can return `OpenError::Io`

**Finding**: `apply_open_document` at `open.rs:56` returns `Result<..., OpenError>` which includes `Io(String)`. Behavior 25 ("open_workspace() shows error toast on load failure") tests the action layer (toast), not the calc layer. No scenario directly tests `apply_open_document` returning `Err(OpenError::Io(_))`.

**Note**: This may be intentional if `apply_open_document` can only return `Io` from `fs::read_to_string` on an unreadable file, which would be a permission/path error. If this error path is unreachable in practice, this is a MINOR-0.

### MINOR-3: Max document size boundary not tested

**Location**: Open Question 6, Section 10

**Finding**: Open Question 6 asks "Is there a hard limit on document size?" but no planned boundary test for max document size exists in the plan. If there IS a limit, this should be tested. If there is NO limit, the Open Question should note this is by design.

---

## MANDATE

To achieve APPROVED status, the following must be added to the test plan:

1. **BDD scenario for `TempFileError`** (MAJOR-1)
   - Required test name: `fn apply_save_document_returns_io_error_when_temp_file_creation_fails()`
   - Must show what input triggers the temp file failure path
   - Must assert the exact error variant and message pattern

2. **BDD scenario for `AtomicRenameError`** (MAJOR-2)
   - Required test name: `fn apply_save_document_returns_io_error_when_atomic_rename_fails()`
   - Must show what input triggers the atomic rename failure path
   - Must assert the exact error variant and message pattern

3. **Clarify or test `apply_open_document` Io error path** (MINOR-2)
   - Either add scenario for `Err(OpenError::Io(_))` at calc layer, OR
   - Note that this error path is unreachable by design

4. **Address empty string boundary for `apply_import_contents`** (MINOR-1)
   - Add to boundary list or add explicit scenario

5. **Close Open Question 6** (max document size)
   - If limit exists: add boundary test
   - If no limit: note "no hard limit by design"

---

## What Was Fixed Well

1. **Behavior 14**: String check removed, wildcard used correctly ✓
2. **Schema violations body**: BDD scenario body added for Behavior 31 ✓
3. **Named functions**: TempFileError and AtomicRenameError have named functions in section 9 ✓
4. **Open Questions**: Section 10 added with 9 known limitations ✓

The fixes improved the plan. The remaining gap is that **named functions exist but no BDD scenario bodies** describe what these error variants mean in the context of `apply_save_document`.

---

*Review completed by test-reviewer skill. Resubmit for full re-review after fixes.*
