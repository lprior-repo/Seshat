# Code Review Defects

STATUS: REJECTED

The implementation was subjected to the 5-phase Black Hat Review. While the previous TOCTOU and control character defects were addressed, **NEW critical defects were discovered**.

---

## PHASE 1: Contract & Bead Parity

### DEFECT 1.1: Inconsistent Max Length Validation (CRITICAL)

**Location:**
- `canvas_domain/src/interaction_reducer/commit.rs:58` - `MAX = 1000`
- `diagram_models/src/projection/ops/edge_ops.rs:17` - `MAX_LABEL_LENGTH = 4096`

**Problem:**
Two different validation functions exist for edge labels with INCONSISTENT maximum lengths:
- `commit.rs::is_valid_label()` rejects labels > 1000 characters
- `edge_ops.rs::is_valid_edge_label()` rejects labels > 4096 characters

**Impact:**
A label that passes validation in one layer (4096 chars) will fail in another (1000 chars). This creates non-deterministic behavior depending on which validation path is taken.

**Fix:**
Consolidate to a SINGLE validation function with a SINGLE constant. Define `MAX_LABEL_LENGTH` in a shared location (e.g., `diagram_models/src/validation.rs`) and use it everywhere.

---

### DEFECT 1.2: Duplicate Validation Logic (HIGH)

**Location:**
- `canvas_domain/src/interaction_reducer/commit.rs:57-73` - `is_valid_label()`
- `diagram_models/src/projection/ops/edge_ops.rs:28-51` - `is_valid_edge_label()`

**Problem:**
Near-identical validation logic exists in TWO places. The functions differ in:
1. Max length (1000 vs 4096)
2. Explicit null byte check (edge_ops has it, commit.rs doesn't)

**Impact:**
- Maintenance nightmare: updates to one will likely miss the other
- Behavioral inconsistency: subtle differences cause edge case bugs
- Violates DRY principle

**Fix:**
Extract validation to a single canonical location. Suggest `diagram_models/src/validation/label.rs` with:
```rust
pub fn is_valid_label(label: &str) -> bool { ... }
pub const MAX_LABEL_LENGTH: usize = 1000; // or 4096, pick ONE
```

---

### DEFECT 1.3: Error Taxonomy Mismatch (MEDIUM)

**Contract specifies:**
```
Error Taxonomy:
- TargetNotFound - The specified edge_id does not exist
- UpdateFailed - The system failed to persist the new label
```

**Implementation provides:**
```rust
// types.rs
pub enum LabelEditError {
    TargetNotFound,      // ✓ Matches
    ValidationError,     // ✗ Not in contract
}

pub enum CommitError {
    LabelEdit(LabelEditError),
    DispatchFailed(DispatchError),  // ✗ Called "UpdateFailed" in contract
}
```

**Impact:**
Confusing API for consumers. Contract says `UpdateFailed` but code has `DispatchFailed`.

**Fix:**
Either update the contract to match implementation, or rename `DispatchFailed` to `UpdateFailed`.

---

## PHASE 2: Farley Engineering Rigor

### PASSED - No defects found

- All functions under 25 lines ✓
- No function exceeds 5 parameters ✓
- Functional core / imperative shell separation maintained ✓
- Tests assert behavior (WHAT), not implementation (HOW) ✓

---

## PHASE 3: NASA-Level Functional Rust

### DEFECT 3.1: Parse, Don't Validate (MEDIUM)

**Location:** `commit.rs:57-73`, `edge_ops.rs:28-51`

**Problem:**
Validation is performed via runtime boolean checks, not type-level parsing. A `ValidatedLabel` newtype should encapsulate these rules:

```rust
// BETTER: Parse at boundary
pub struct ValidatedLabel(String);

impl ValidatedLabel {
    pub fn parse(s: String) -> Result<Self, LabelValidationError> {
        if s.len() > MAX_LABEL_LENGTH { return Err(TooLong); }
        // ... other checks
        Ok(Self(s))
    }
}
```

**Impact:**
Current approach requires re-validation at multiple points. Type-level parsing would guarantee validity at compile time.

**Fix:**
Introduce `ValidatedLabel` newtype for edge/node labels.

---

## PHASE 4: Ruthless Simplicity & DDD

### DEFECT 4.1: Quality Gates Disabled (HIGH)

**Location:** `diagram_models/src/projection/ops/edge_ops.rs:6-7`

```rust
#![allow(dead_code)]
#![allow(unused_imports)]
```

**Problem:**
These attributes DISABLE the compiler's quality gates. Dead code and unused imports are being hidden rather than fixed.

**Impact:**
- Dead code accumulates
- Unused imports suggest incomplete refactoring
- Hides real problems

**Fix:**
Remove these attributes. Fix the underlying issues (remove dead code, remove unused imports).

---

### DEFECT 4.2: Mutable Variable Without Need (LOW)

**Location:** `commit.rs:89`, `commit.rs:198`

```rust
let mut old_label = String::new();
// ... closure mutates old_label
```

**Problem:**
`old_label` is mutated from within the closure via external capture. This works but is less clean than returning a tuple.

**Impact:**
Minor readability/maintainability concern.

**Fix:**
Return `(DiagramDocument, String)` tuple from closure instead of mutating external variable.

---

## PHASE 5: The Bitter Truth

### DEFECT 5.1: Clever Inconsistency (MEDIUM)

**Problem:**
Having two max lengths (1000 and 4096) for the same domain concept is confusing. This reeks of "two different developers made assumptions independently."

**Sniff Test Result:** FAIL

A junior developer looking at this code would be confused: "Why are there two validation functions? Which one is correct? Why are the limits different?"

---

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 1 |
| HIGH | 2 |
| MEDIUM | 3 |
| LOW | 1 |

## Mandatory Fixes Before Approval

1. **[CRITICAL]** Consolidate max length to a single value (pick 1000 or 4096, document the choice)
2. **[HIGH]** Extract validation to a single canonical function in a shared module
3. **[HIGH]** Remove `#![allow(dead_code)]` and `#![allow(unused_imports)]` from edge_ops.rs
4. **[MEDIUM]** Align error taxonomy with contract OR update contract to match implementation

## What Was Fixed (Previous Defects)

✓ TOCTOU race condition - existence checks now INSIDE mutation closure
✓ Control character whitelist - `\n`, `\r`, `\t` now allowed
✓ Visual spoofing protection - zero-width and bidi characters blocked

---

**VERDICT: REJECTED**

The code does not pass the 5-phase review. Fix the critical and high-severity defects and resubmit.
