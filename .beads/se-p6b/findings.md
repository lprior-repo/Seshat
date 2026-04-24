# BLACK HAT SECURITY AUDIT — se-p6b (wave3-8)

**Date**: 2026-04-24
**Auditor**: guzzle (polecat)
**Scope**: Security audit of Seshat codebase
**Files Reviewed**: 15+ source files across diagram_tool, diagram_models, canvas_domain

---

## EXECUTIVE SUMMARY

| Category | Status |
|----------|--------|
| Unsafe Code | ✅ PASS — `#![forbid(unsafe_code)]` enforced everywhere |
| SQL Injection | ✅ PASS — All queries use parameterized `?N` bindings |
| Input Validation | ✅ MOSTLY PASS — Strong validation on most vectors |
| Path Traversal | ⚠️ ISSUE — `validate_file_path_format` doesn't check `../` |
| Error Leakage | ✅ PASS — Errors converted to generic JSON strings |
| Unused Code | ⚠️ ISSUE — Patch command has dead code paths |
| Authentication | N/A — Local CLI tool, no auth needed |

---

## ENVIRONMENT CHECKS

| Check | Result |
|-------|--------|
| `#![forbid(unsafe_code)]` present | ✅ PASS — All crates enforce this |
| No `unwrap()` in source | ✅ PASS — `#![cfg_attr(not(test), deny(clippy::unwrap_used))]` |
| No `panic!()` in source | ✅ PASS — Enforced via deny attribute |
| Parameterized SQL queries | ✅ PASS — Uses `?1, ?2, etc.` bindings |
| Input validation present | ✅ PASS — Multiple validation layers |
| No hardcoded secrets | ✅ PASS — No credentials/secrets found |

---

## SECURITY ANALYSIS BY MODULE

### 1. Physical I/O (diagram_models/src/physical_io.rs)

**Strengths:**
- Recursion depth limit of 100 prevents stack exhaustion
- Validates non-finite floats (NaN, Infinity)
- Validates JSON structure before deserialization

**Issues Found:**

| # | Severity | Description |
|---|----------|-------------|
| PIO-1 | LOW | `File::create(path)` and `File::open(path)` accept relative paths. No path canonicalization or traversal check. A malicious file path like `../../etc/passwd` could be passed. However, OS permissions prevent writing outside user's access. |

---

### 2. AI Document Validation (diagram_models/src/schema_ai_documents/)

**Strengths:**
- `id` and `key` must be non-empty after trimming
- `location_data` format validated per `location_type`:
  - GPS: lat/lon range validation (-90/90, -180/180)
  - FilePath: no null bytes, 4096 char limit
  - DocumentPosition: line:col format
  - URL: requires http:// or https:// prefix

**Issues Found:**

| # | Severity | Description |
|---|----------|-------------|
| AID-1 | LOW | `validate_file_path_format` doesn't check for path traversal patterns (`../`). A malicious input like `../../shadow` could pass validation. However, this is stored data, not used for actual file access in current implementation. |
| AID-2 | INFO | `validate_url_format` only checks for `http://` or `https://` prefix and that there's a `.` in the host. Doesn't restrict to safe hosts. But since URLs are stored (not fetched), this is low risk. |

---

### 3. Server Functions (diagram_tool/src/server/ai_documents.rs)

**Strengths:**
- `#![forbid(unsafe_code)]` enforced
- Errors converted to generic JSON strings — no internal detail leakage
- LocationType parsed via `LocationType::from_str` with validation
- AiDocument created via `AiDocument::new()` with full validation

**Issues Found:** None

---

### 4. Store Async (diagram_tool/src/store_async/ai_documents.rs)

**Strengths:**
- All SQL uses parameterized queries
- `handle_insert_error` catches duplicate key violations specifically
- Row mapping uses dedicated struct `AiDocumentRow`

**Issues Found:** None

---

### 5. CLI Patch Command (diagram_tool/src/cli/patch.rs)

**Critical Issues:**

| # | Severity | Description |
|---|----------|-------------|
| PATCH-1 | **HIGH** | `execute_inner` reads `patch_content` from file but **never uses it**. The `_patch_content` variable is read and dropped. This is dead code — the patch is loaded but not applied. The output path is also never used. **The patch command is incomplete/broken.** |
| PATCH-2 | LOW | Path parameters (`input`, `patch`, `output`) are passed directly to `std::fs::read_to_string` and `physical_io::load_document` without traversal checks. |

---

### 6. Validation Rules (diagram_models/src/validation/rules.rs)

**Strengths:**
- Comprehensive document structure validation
- DAG property enforced (no cycles)
- Edge references validated (no dangling references)
- Numeric ranges enforced (thickness >= 0, label_offset_t in [0,1])
- Hex color format validated

**Issues Found:** None

---

### 7. Clippy/CI Compliance

**Status:**
```
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
```

All crates enforce strict clippy rules in non-test code. This prevents accidental introduction of unsafe patterns.

---

## ISSUE SUMMARY

| ID | Severity | Module | Description |
|----|----------|--------|-------------|
| PIO-1 | LOW | physical_io | Relative path handling without traversal check |
| AID-1 | LOW | schema_ai_documents | File path validation misses `../` traversal |
| AID-2 | INFO | schema_ai_documents | URL validation is permissive |
| PATCH-1 | **HIGH** | cli/patch | Patch command reads but never applies patch (incomplete) |
| PATCH-2 | LOW | cli/patch | Path parameters lack traversal sanitization |

---

## REMEDIATION RECOMMENDATIONS

### PATCH-1 (HIGH) — Patch Command Incomplete

The `PatchCommand::execute_inner` method reads the patch file but never applies it:

```rust
fn execute_inner(&self) -> Result<(), PatchError> {
    let _doc = physical_io::load_document(&self.input)?;
    let _patch_content = std::fs::read_to_string(&self.patch)?; // READ BUT NEVER USED
    let _output_path = &self.output; // NEVER USED
    Ok(())
}
```

**Fix**: Either implement the patch application logic or remove the command entirely. Dead code is a maintenance liability.

### PATCH-2 / PIO-1 / AID-1 — Path Traversal

Consider canonicalizing paths before use:

```rust
use std::path::Path;

fn canonicalize_path(path: &Path) -> Result<std::path::PathBuf, Error> {
    path.canonicalize()
        .map_err(|e| Error::IoError(e))
}
```

This resolves `../` sequences and ensures the final path is within the expected directory.

---

## CONCLUSION

**Overall Security Posture**: GOOD

The codebase demonstrates strong security practices:
- `#![forbid(unsafe_code)]` everywhere
- Parameterized SQL queries prevent injection
- Comprehensive input validation
- Error messages don't leak internals

**Main Concern**: The patch command is incomplete (PATCH-1) — it reads a patch file but never applies it. This is a functional bug, not just a security issue.

**Recommendation**: Fix PATCH-1 (implement patch application or remove command). Consider path canonicalization for defense in depth, but current risk is LOW due to OS-level file permissions.

---

## VERDICT: **APPROVED WITH NOTES**

The security audit passed. The patch command incompleteness is a functional defect, not a security vulnerability per se, but it should be addressed.

**Action Items:**
1. Fix or remove `PatchCommand::execute_inner` (PATCH-1)
2. Consider path canonicalization in file I/O operations
3. The other findings are low-risk and acceptable for a local CLI tool