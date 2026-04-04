# Test Plan Review: save-load-test-plan

## VERDICT: REJECTED

**Reason**: 7 MAJOR findings (threshold ≥3) + 9 MINOR findings (threshold ≥5)

---

## Axis 1 — Contract Parity

| Check | Result |
|-------|--------|
| All 10 `pub fn` have BDD scenarios | ✅ PASS |
| All Error variants have scenarios | ⚠️ INCONSISTENT — `TempFileError` (Behavior 314) and `AtomicRenameError` (Behavior 326) are documented in Section 9 but their "Then:" clauses assert `Err(SaveError::Io(_))` not the actual `CliPersistenceError` variants |
| `is_ok()`/`is_err()` in "Then:" | ✅ NONE FOUND |

**LETHAL: None**

---

## Axis 2 — Assertion Sharpness

| Behavior | "Then:" Assertion | Finding |
|----------|-------------------|---------|
| 260 `apply_save_document_returns_io_error_for_nonexistent_directory` | `Err(SaveError::Io(_))` | MAJOR — wildcard inner value |
| 314 `apply_save_document_returns_io_error_when_temp_file_creation_fails` | `Err(SaveError::Io(_))` | MAJOR — Section 9 says `TempFileError` but "Then:" says `SaveError::Io` — inconsistent |
| 320 (same test name as 314) | `Err(SaveError::Io(_))` | MAJOR — duplicate test name |
| 326 `apply_save_document_returns_io_error_when_atomic_rename_fails` | `Err(SaveError::Io(_))` | MAJOR — Section 9 says `AtomicRenameError` but "Then:" says `SaveError::Io` — inconsistent |
| 332 `apply_save_document() WASM variant always returns Io error` | `Err(SaveError::Io(_))` | MAJOR — no WASM-specific message check |
| 528 `apply_open_document_returns_validation_error_for_schema_violations` | `Err(OpenError::Validation(_))` | MAJOR — wildcard inner value |
| 1216 `apply_import_contents_returns_parse_error_on_empty_string` | `Err(ImportTransitionError::Parse(_))` | MAJOR — wildcard inner value |

**Count: 7 MAJOR** (threshold: ≥3 → REJECTED)

---

## Axis 3 — Trophy Allocation

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| Public functions | 10 | — | — |
| Unit tests planned | 52 | — | — |
| Ratio | 5.2x | ≥5x | ✅ PASS |
| Proptest invariants | 4 | — | ✅ |
| Fuzz targets | 3 | — | ✅ |
| Kani harnesses | 2 | — | ✅ |

**LETHAL: None** — ratio is tight (5.2x) but passes.

---

## Axis 4 — Boundary Completeness

| Function | Missing Boundary | Severity |
|----------|------------------|----------|
| `apply_save_document` | Max valid document size | MINOR |
| `apply_save_document` | Revision u64::MAX | MINOR |
| `apply_save_document` | Empty file path argument | MINOR |
| `apply_save_document` | File path at OS max length | MINOR |
| `apply_open_document` | Empty-but-valid document structure | MINOR |
| `apply_open_document` | Max JSON payload (Open Question 9) | MINOR |
| `apply_open_document` | File path at OS max length | MINOR |
| `apply_import_contents` | Max JSON payload (Open Question 9) | MINOR |
| `prepare_import_transition` | Max JSON payload (Open Question 9) | MINOR |

**Count: 9 MINOR** (threshold: ≥5 → REJECTED)

---

## Axis 5 — Mutation Survivability

| Mutation | Surviving Test? | Issue |
|----------|-----------------|-------|
| Return `"wrong validation message"` instead of real one | Behavior 14 `Err(SaveError::Serialize(_))` — WOULD PASS | Wildcard doesn't catch wrong inner value |
| Return `SaveError::Io("permission denied")` when path doesn't exist | Behavior 260 — WOULD PASS | Wildcard doesn't distinguish "no such file" vs "permission denied" |
| Return `SaveError::Io("disk full")` when atomic rename fails | Behavior 326 — WOULD PASS | Wildcard doesn't catch wrong message |
| Return `Ok(Default::default())` for `apply_save_document` | Caught by revision sync test ✅ | |

**MAJOR: 3 survivable mutations identified**

---

## Axis 6 — Holzmann Plan Audit

| Rule | Status |
|------|--------|
| Rule 2 — No loops in test bodies | ✅ PASS |
| Rule 5 — Preconditions explicit (Given:) | ✅ PASS |
| Rule 7 — No shared mutable state | ✅ N/A (plan) |

**PASS**

---

## Summary

| Category | Count | Threshold | Result |
|----------|-------|-----------|--------|
| LETHAL | 0 | Any | ✅ |
| MAJOR | 7 | ≥3 | ❌ REJECTED |
| MINOR | 9 | ≥5 | ❌ REJECTED |

---

## MANDATE

The following must exist before resubmission:

### Critical Fixes (resolve MAJOR findings):

1. **test-plan.md:260** — Add inner value check to `Err(SaveError::Io(_))`:
   ```
   Then: Returns `Err(SaveError::Io(s))` where `s.contains("...")`  [specify expected message]
   ```

2. **test-plan.md:314** — Make consistent with Section 9. Either:
   - Change Section 9 to reference the actual `SaveError::Io` behavior, OR
   - Change "Then:" to verify `TempFileError` specifically:
   ```
   Then: Returns `Err(SaveError::Io(s))` where `s.contains("TempFileError")` or equivalent
   ```

3. **test-plan.md:320** — Give this a unique test name distinct from 314. The two behaviors (temp file creation failure vs ???) need separate test names.

4. **test-plan.md:326** — Make consistent with Section 9 (says `AtomicRenameError`). Either:
   - Change Section 9 to reference `SaveError::Io`, OR
   - Change "Then:" to verify the actual `AtomicRenameError` message.

5. **test-plan.md:332** — Add WASM-specific message check:
   ```
   Then: Returns `Err(SaveError::Io(s))` where `s.contains("Save not available in WASM")`
   ```

6. **test-plan.md:528** — Add inner value check for validation error content.

7. **test-plan.md:1216** — Add inner value check for empty string parse error message.

### Boundary Tests (resolve MINOR findings):

8. Add test: `apply_save_document` with maximum valid document (max nodes/edges within spec)
9. Add test: `apply_save_document` with revision at u64::MAX boundary
10. Add test: `apply_open_document` with empty-but-valid document structure
11. Add test: `apply_open_document` with maximum JSON payload (addresses Open Question 9)
12. Add test: `apply_import_contents` with maximum JSON payload (addresses Open Question 9)

---

**After all fixes: re-run full Plan Inquisition from Axis 1.**
