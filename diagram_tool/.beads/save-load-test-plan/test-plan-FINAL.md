# Test Plan Review — save-load-test-plan (FINAL)

**Date**: 2026-04-04
**Mode**: Plan Inquisition (contract.md + test-plan.md)
**Status**: **APPROVED**

---

## Executive Summary

All 3 previously identified defects have been fixed:
1. ✓ `OpenError::Io` removed from `apply_open_document` coverage table (N/A — IO errors arise at action layer only)
2. ✓ Behaviors 35 and 36 now use `==` assertions with stored original count
3. ✓ CliPersistenceError → SaveError mapping added to contract.md

The plan has 46 behaviors across 10 public functions with excellent coverage ratios (5.2x unit test density), proper error variant assertions, and comprehensive mutation checkpoints.

---

## Axis 1 — Contract Parity

| # | Function | Location | Behaviors | Status |
|---|----------|----------|-----------|--------|
| 1 | `apply_save_document` (native) | save.rs:32 | 9-16, 314, 320 | ✓ |
| 2 | `apply_save_document` (WASM) | save.rs:54 | 16 | ✓ |
| 3 | `save_workspace` | save.rs:62 | 1-8 | ✓ |
| 4 | `apply_open_document` | open.rs:56 | 29-31 | ✓ |
| 5 | `open_workspace` | open.rs:296 | 17-28 | ✓ |
| 6 | `prepare_import_transition` | common.rs:14 | 37-42 | ✓ |
| 7 | `apply_import_contents` | common.rs:39 | 34-36 | ✓ |
| 8 | `update_load_save_success` | common.rs:55 | 32 | ✓ |
| 9 | `update_load_save_error` | common.rs:64 | 33 | ✓ |
| 10 | `use_global_keyboard` | keyboard.rs:21,181 | 43-45 | ✓ |

**10/10 functions have BDD scenarios. ✓**

### Error Enum Coverage

| Enum | Variant | Covered By | Assertion Type | Status |
|------|---------|------------|----------------|--------|
| SaveError | NoFilePath | Behavior 13 | Exact variant | ✓ |
| SaveError | Serialize(String) | Behavior 14 | Wildcard (validation message) | ✓ |
| SaveError | Io(String) | Behaviors 12, 15, 314, 320 | Wildcard (OS messages) | ✓ |
| OpenError | Parse(String) | Behaviors 30, 31 | Wildcard + `.contains("version")` | ✓ |
| OpenError | Validation(String) | Behavior 31 | Wildcard + `.contains("validation")` | ✓ |
| OpenError | Io(String) | N/A | Not applicable — IO errors at action layer | ✓ |
| ImportTransitionError | Parse(String) | Behavior 35 | Wildcard + `.len() > 0` | ✓ |
| ImportTransitionError | Validation(String) | Behavior 36 | Wildcard + `.len() > 0` | ✓ |
| CliPersistenceError | IoError | cli_persistence/tests.rs | Existing | ✓ |
| CliPersistenceError | ParseError | cli_persistence/tests.rs | Existing | ✓ |
| CliPersistenceError | ValidationError | cli_persistence/tests.rs | Existing | ✓ |
| CliPersistenceError | TempFileError | Behavior 314 | Wildcard (OS messages) | ✓ |
| CliPersistenceError | AtomicRenameError | Behavior 320 | Wildcard (OS messages) | ✓ |
| CliPersistenceError | NoValidDocument | Behaviors 26, integration | Existing | ✓ |
| CliPersistenceError | PathTraversalDenied | Behavior 15 | Wildcard + `.contains("Path traversal denied")` | ✓ |

**All error variants covered with documented rationale or existing tests. ✓**

---

## Axis 2 — Assertion Sharpness

| Behavior | Then Clause | Assessment |
|----------|------------|------------|
| 9 | `is_dirty() == false`, `last_saved_revision() == Revision(10)` | Concrete values ✓ |
| 10 | `file_path() == Some(PathBuf::from("/original.json"))` | Concrete value ✓ |
| 12 | `Err(SaveError::Io(_))` | Wildcard acceptable (OS message) ✓ |
| 13 | `Err(SaveError::NoFilePath)` | Exact variant ✓ |
| 14 | `Err(SaveError::Serialize(_))` | Wildcard acceptable (validation) ✓ |
| 15 | `Err(SaveError::Io(s)) where s.contains("Path traversal denied")` | Concrete string check ✓ |
| 29 | `next_doc.nodes.len() == 3`, `next_doc.edges.len() == 2` | Concrete values ✓ |
| 30 | `Err(OpenError::Parse(s)) where s.len() > 0` | Concrete check ✓ |
| 31 | `Err(OpenError::Validation(s)) where s.contains("validation")` | Concrete check ✓ |
| **35** | `doc.nodes.len() == 3` with stored original count | **FIXED** — concrete `==` ✓ |
| **36** | `doc.nodes.len() == 3` with stored original count | **FIXED** — concrete `==` ✓ |
| 37 | `font_size == 14`, `dag_rank == 7` | Concrete values ✓ |
| 38 | `icon_url == "/resources/{icon}"` | Concrete value ✓ |
| 39 | `icon_url == "/custom/path"` | Concrete value ✓ |
| 40 | `arrow_type == ArrowType::Step`, `ArrowType::Straight` | Concrete enum values ✓ |
| 314 | `Err(SaveError::Io(_))` with note on OS unpredictability | Wildcard acceptable ✓ |
| 320 | `Err(SaveError::Io(_))` with note on OS unpredictability | Wildcard acceptable ✓ |

**No `is_ok()` or `is_err()` as sole assertions. ✓**

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

| Function | Boundaries Named | Status |
|----------|-----------------|--------|
| `apply_save_document` | Min path, invalid path, path traversal, no session path, WASM, temp file failure, atomic rename failure | ✓ |
| `apply_open_document` | Invalid JSON, missing version, schema violations | ✓ (IO errors N/A at calc layer) |
| `apply_import_contents` | Valid JSON, parse error, validation error, empty string (Open Question 10) | ✓ |
| `prepare_import_transition` | Legacy fields, icon migration, arrow types | ✓ |

**Known limitations documented**:
- Max document size: Open Question 6, marked "not in scope" (bounded by memory)
- Revision overflow (u64::MAX): Open Question 7, acknowledged

---

## Axis 5 — Mutation Survivability

### Critical Mutations Coverage

| Mutation | Caught By | Status |
|----------|-----------|--------|
| Remove `is_dirty()` check | `apply_save_document_clears_dirty_flag_on_success` | ✓ |
| Skip `mark_saved()` call | `apply_save_document_syncs_revision_from_current_document` | ✓ |
| Wrong revision assignment | `apply_save_document_syncs_revision_from_current_document` | ✓ |
| Map `CliPersistenceError::PathTraversalDenied` to wrong variant | `apply_save_document_returns_path_traversal_error_when_path_contains_double_dot` | ✓ |
| **TempFileError mapping** | `apply_save_document_returns_io_error_when_temp_file_creation_fails` | **FIXED** ✓ |
| **AtomicRenameError mapping** | `apply_save_document_returns_io_error_when_atomic_rename_fails` | **FIXED** ✓ |
| Skip history push | `apply_import_contents_updates_doc_and_history_atomically` | ✓ |
| Skip error rollback | `apply_import_contents_leaves_doc_and_history_unchanged_on_parse_error` | ✓ |

**All critical mutations have named test functions with BDD scenario bodies. ✓**

---

## Axis 6 — Holzmann Plan Audit

| Rule | Assessment |
|------|------------|
| Rule 1 (Linear) | All BDD scenarios have linear Given→When→Then ✓ |
| Rule 2 (Bound loops) | No loops in plan text ✓ |
| Rule 3 (Resource cleanup) | N/A for plan (enforced in implementation) ✓ |
| Rule 4 (One job) | Each scenario tests one behavior ✓ |
| Rule 5 (Assumptions explicit) | Given clauses explicitly state preconditions ✓ |
| Rule 6 (No swallowed errors) | Assertions use exact variants or wildcards with content checks ✓ |
| Rule 7 (Narrow state) | N/A for plan ✓ |
| Rule 8 (Side effects surfaced) | Test names describe the action clearly ✓ |
| Rule 9 (One layer magic) | All scenarios self-contained ✓ |

---

## Verdict

### Severity Tally

| Severity | Count | Threshold | Action |
|----------|-------|-----------|--------|
| LETHAL | 0 | Any | — |
| MAJOR | 0 | ≥3 | — |
| MINOR | 0 | ≥5 | — |

**Status**: 0 LETHAL + 0 MAJOR + 0 MINOR → **APPROVED**

---

## LETHAL FINDINGS

None.

---

## MAJOR FINDINGS

None.

---

## MINOR FINDINGS

None.

---

## Confirmed Fixes

### Fix 1: OpenError::Io Removed from apply_open_document Coverage ✓

**Location**: contract.md line 90, test-plan.md Section 9 (OpenError table)

**Before**: OpenError::Io was listed without clear rationale for why it doesn't apply to `apply_open_document`

**After**: 
- contract.md explicitly states "IO errors arise only at the `open_workspace` action layer"
- test-plan.md Section 9 OpenError table shows "Not applicable" with explanation
- This is correct because `apply_open_document` takes `&str` content, not a file path — IO errors occur at the action layer where file reading happens

### Fix 2: Behaviors 35 and 36 Use Concrete == Assertions ✓

**Location**: test-plan.md lines 584-608

**Before**: Plan showed vague "doc.nodes.len() == original count" without specifying concrete value

**After**:
- Behavior 35 (line 590): `Then: doc.nodes.len() == 3 and history.can_undo() == true (unchanged from before call)`
- Note explicitly states: "Store original count before call: `let original_node_count = doc.nodes.len();` then assert `assert_eq!(doc.nodes.len(), original_node_count);`"
- Same pattern for Behavior 36

### Fix 3: CliPersistenceError → SaveError Mapping Added to contract.md ✓

**Location**: contract.md lines 82-95

**Added**:
```markdown
## Error Mapping (CliPersistenceError → SaveError)

The `apply_save_document()` function calls `save_workspace_atomic()` which returns `CliPersistenceError`. These errors are mapped to `SaveError` variants:

| CliPersistenceError | Maps To | Rationale |
|---------------------|---------|-----------|
| IoError(_) | SaveError::Io(_) | OS-level I/O failures |
| ParseError(_) | SaveError::Serialize(_) | JSON serialization failures |
| ValidationError(_) | SaveError::Serialize(_) | Document validation failures |
| TempFileError(_) | SaveError::Io(_) | Temp file creation failures are I/O |
| AtomicRenameError{..} | SaveError::Io(_) | Atomic rename failures are I/O |
| NoValidDocument(_) | Not reachable | Only used during load, not save |
| PathTraversalDenied{..} | SaveError::Io(_) | Path traversal rejected before I/O |
```

---

## Recommendation

**STATUS: APPROVED**

The test plan is complete and ready for implementation. All public functions have BDD scenarios with concrete assertions. All error variants are covered or explicitly documented as N/A with rationale. The trophy allocation meets the 5x density target. Mutation checkpoints are comprehensive.

### Implementation Priority

1. **Unit tests** (52): Focus on Calc layer — `apply_save_document`, `apply_open_document`, `prepare_import_transition`, `apply_import_contents`
2. **Integration tests** (16): Component interactions with FileDialog, store_bridge, signals
3. **Proptest invariants** (4): Revision sync, file path preservation, atomicity, round-trip
4. **Fuzz targets** (3): `parse_diagram_document_with_compat`, `save_workspace_atomic`, `apply_import_contents`
5. **Kani harnesses** (2): Path traversal prevention, atomicity on error

---

*Test plan approved by test-reviewer skill. Ready for implementation phase.*
