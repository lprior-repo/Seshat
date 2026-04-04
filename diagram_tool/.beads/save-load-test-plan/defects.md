# Adversarial Code Review: File Save/Load Persistence

**Reviewer**: black-hat-reviewer  
**Date**: 2026-04-04  
**Bead**: save-load-test-plan  
**Files Reviewed**:
- `diagram_tool/src/cli_persistence/mod.rs`
- `diagram_tool/src/cli_persistence/write.rs`
- `diagram_tool/src/cli_persistence/read.rs`
- `diagram_tool/src/ui/toolbar/persistence/save.rs`
- `diagram_tool/src/ui/toolbar/persistence/open.rs`
- `diagram_tool/src/ui/toolbar/persistence/common.rs`
- `diagram_tool/src/hooks/keyboard.rs`

---

## PHASE 1: Contract & Bead Parity — FAIL

### CRITICAL: Keyboard Shortcut Contract Violation

**Contract** (`contract.md:24`):
> `use_global_keyboard` — "Handles Ctrl+S + Ctrl+O" (native), "Handles Ctrl+S only (no 'o' key in handler at line 215)" (WASM)

**Actual** (`hooks/keyboard.rs:41-50`):
```rust
const handled = modifier && (
    key === 'z' ||
    key === 'y' ||
    key === 'a' ||
    key === 'c' ||
    key === 'x' ||
    key === 'v' ||
    key === 'd' ||
    key === 'g'
);
```

**VERDICT**: `use_global_keyboard` handles Ctrl+Z, Ctrl+Y, Ctrl+A, Ctrl+C, Ctrl+X, Ctrl+V, Ctrl+D, Ctrl+G. It does NOT handle Ctrl+S or Ctrl+O. The contract is factually wrong. The comment about "line 215" is meaningless—there is no line 215 in the file and no WASM-specific 'o' key handling.

---

## PHASE 2: Farley Engineering Rigor — FAIL

### LETHAL-1: `validate_safe_path` is 82 lines (limit: 25)

**Location**: `cli_persistence/mod.rs:66-149`

This function has 6+ levels of nested conditionals with multiple early returns. It is a "GOTO farm" disguised as defensive programming. The path traversal logic cannot be verified by inspection.

```rust
// Line 66: 82-line function
pub fn validate_safe_path(path: &Path, base_dir: &Path) -> Result<PathBuf, CliPersistenceError> {
    // 5 nested if/else blocks with early returns
    // Multiple ? operators through fallible canonicalize calls
    // Line 109: complex AND/OR condition
}
```

**Rule**: Any function over 25 lines MUST be rejected. Rewrite immediately.

### LETHAL-2: `save_workspace` is 87 lines (limit: 25)

**Location**: `persistence/save.rs:62-149`

The `save_workspace` action function is 87 lines. It contains:
- Toast creation (lines 67-70)
- WASM vs native branching (lines 71-148)
- Nested match arms with inner matches
- Async task spawning

This function is doing too much. It should be split into smaller helper functions.

### MAJOR-1: Path Traversal Protection NOT Enforced in `apply_save_document`

**Location**: `persistence/save.rs:32-51`

The `apply_save_document` function calls `save_workspace_atomic` directly:
```rust
save_workspace_atomic(doc, file_path).map_err(...)
```

It does NOT call `validate_safe_path`. The `validate_safe_path` function exists (82 lines!) but is never used in the save path. This means:
- Path traversal attacks are NOT blocked by `apply_save_document`
- A malicious path like `../../../etc/passwd` could be passed to `save_workspace_atomic`

**However**: `save_workspace_atomic` creates a temp file in the parent directory of the target, so it cannot escape to arbitrary locations. But the `validate_safe_path` contract is to "make path traversal impossible" — and it's not being called.

---

## PHASE 3: NASA-Level Functional Rust (Big 6) — PARTIAL FAIL

### MAJOR-2: `mod.rs:136` uses `.unwrap()` on fallible operation

**Location**: `cli_persistence/mod.rs:136`
```rust
let canonical_base = std::fs::canonicalize(base_dir)?;
```

Wait, this uses `?` not `.unwrap()`. Let me re-read...

Actually, line 136 uses `?`. But the earlier logic at lines 85-132 has complex branching that returns early in many cases. The function is complex but doesn't use `.unwrap()`.

**Correction**: No `.unwrap()` found in this function. But the complex branching IS a panic vector because incorrect logic could cause the wrong branch to be taken.

### MAJOR-3: Silent Error Swallowing in Toast Functions

**Location**: `persistence/common.rs:55-71`
```rust
pub fn update_load_save_success(toast_handle: ToastHandle, title: &str, detail: String) {
    let _ = toast_handle.update(ToastUpdate {  // SILENTLY IGNORES Result
        title: Some(title.to_string()),
        detail: Some(Some(detail)),
        intent: Some(ToastIntent::Success),
        action: None,
    });
}

pub fn update_load_save_error(toast_handle: ToastHandle, title: &str, detail: String) {
    let _ = toast_handle.update(ToastUpdate {  // SILENTLY IGNORES Result
        title: Some(title.to_string()),
        detail: Some(Some(detail)),
        intent: Some(ToastIntent::Error),
        action: None,
    });
}
```

**Rule Violation**: "Flag EVERY `unwrap()`, `expect()`, `panic!()`, or unnecessary `let mut`" — but `let _ = expr` that ignores a `Result` is equally bad. If the toast update fails, the user gets NO feedback about save/load success or failure.

### MAJOR-4: `common.rs:51` uses `Err(err)` pattern masking error identity

**Location**: `persistence/common.rs:51`
```rust
Err(err) => Err(err),
```

This passes through the error unchanged, masking which variant of `ImportTransitionError` occurred. While semantically equivalent, it suggests the author wasn't thinking about error identity.

---

## PHASE 4: Ruthless Simplicity & DDD — PARTIAL FAIL

### MINOR-1: Dead Code in `write.rs`

**Location**: `cli_persistence/write.rs:58`
```rust
let json_content = to_canonical_pretty_json(doc)?;
// ...
let bytes = json_content.len() as u64;  // computed at line 58
// ... but bytes is NEVER USED
emit_stage_event(
    "persisted",
    &StageDetails::new()
        .with_path(path)
        .with_bytes_written(json_content.len() as u64),  // recomputes instead!
);
```

The variable `bytes` is computed but unused. The code recomputes `json_content.len()` in the `emit_stage_event` call. This is dead code and wasted computation.

### MINOR-2: `#![allow(dead_code)]` on persistence modules

**Location**: `persistence/save.rs:1`, `persistence/open.rs:1`, `persistence/common.rs:1`

All three files start with `#![allow(dead_code)]`. This is suspicious—if the code is unused, delete it. If it's needed, remove the allow.

---

## PHASE 5: The Bitter Truth — PASS

### What Passes

1. **Error types are proper enums** — `SaveError`, `OpenError`, `ImportTransitionError`, `CliPersistenceError` are all well-structured sum types.

2. **Pure calc / I/O separation is correct** — `apply_save_document` and `apply_open_document` are pure calc functions. The I/O is correctly isolated in `save_workspace_atomic`, `load_workspace_with_lkg`, and the async `save_workspace`/`open_workspace` actions.

3. **Atomic write pattern is sound** — Temp file + fsync + atomic rename is the correct pattern for crash-safe writes.

4. **LKG fallback is implemented** — `load_workspace_with_lkg` correctly attempts primary, then LKG fallback.

5. **Tests have proper assertions** — The tests in `save.rs` (lines 169-295) have concrete assertions:
   - `assert!(result.is_ok())`
   - `assert!(!saved_session.is_dirty())`
   - `assert_eq!(saved_session.last_saved_revision(), Revision::new(10))`
   - `assert!(path.exists())`

6. **`cli_persistence/mod.rs` has proper guards** — `#![cfg_attr(not(test), deny(clippy::unwrap_used))]` etc. are correctly applied.

---

## Summary of Defects

| ID | Severity | Location | Issue |
|----|----------|----------|-------|
| 1 | CRITICAL | `hooks/keyboard.rs:41-50` | Contract says Ctrl+S+O handled; implementation handles Z/Y/A/C/X/V/D/G only |
| 2 | LETHAL | `cli_persistence/mod.rs:66-149` | `validate_safe_path` is 82 lines (limit: 25) |
| 3 | LETHAL | `persistence/save.rs:62-149` | `save_workspace` is 87 lines (limit: 25) |
| 4 | MAJOR | `persistence/save.rs:32-51` | Path traversal protection (`validate_safe_path`) not enforced in `apply_save_document` |
| 5 | MAJOR | `persistence/common.rs:55-71` | Silent `let _ =` ignores `Result` from toast updates |
| 6 | MINOR | `cli_persistence/write.rs:58` | Dead code: `bytes` variable computed but unused |
| 7 | MINOR | `persistence/*.rs:1` | `#![allow(dead_code)]` on all persistence modules |

---

## Verdict

**STATUS: REJECTED**

### Evidence

1. **Contract violation**: `use_global_keyboard` does not handle Ctrl+S or Ctrl+O as the contract claims.

2. **Farley constraints violated**:
   - `validate_safe_path`: 82 lines (328% of 25-line limit)
   - `save_workspace`: 87 lines (348% of 25-line limit)

3. **Path traversal protection is unused**: The `validate_safe_path` function exists but is not called by `apply_save_document`, leaving a potential security gap.

4. **Silent error swallowing**: Toast update failures are silently ignored, potentially leaving users without feedback.

### Required Fixes

1. **UPDATE `contract.md`** to accurately describe `use_global_keyboard` behavior (handles Z/Y/A/C/X/V/D/G, NOT S/O).

2. **REFACTOR `validate_safe_path`** into smaller functions (<25 lines each). Current implementation is unverifiable.

3. **REFACTOR `save_workspace`** into smaller functions. Consider splitting WASM/native branches into separate functions.

4. **ADD `validate_safe_path` call** to `apply_save_document` OR document why it's not needed (currently the security benefit is unused).

5. **CHANGE `let _ = toast_handle.update(...)` to proper error handling** or at minimum log the failure.

6. **REMOVE dead code** (`bytes` variable in `write.rs`).

7. **REMOVE `#![allow(dead_code)]`** or justify each occurrence.

---

*This review found 2 LETHAL and 2 MAJOR issues. The code fails Farley constraints and has contract violations. A rewrite is required before this code can be approved.*
