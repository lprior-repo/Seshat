# Code Review Defects: seshat-it8 (nu_runner/src/lib.rs)

## Summary
Implementation REJECTED due to 2 critical flaws across Phase 2 (Farley Rigor) and Phase 4 (DDD Smells).

---

## Defect 1: Function Length Violation (Farley Rigor)

**Severity**: HIGH  
**Phase**: 2 - Farley Rigor

### Description
Two core functions exceed the 25-line hard limit:

1. **`execute()`** (lines 254-283): ~30 lines
   - Contains: I2 check, P1/P2/P3 validation, timeout validation, state management, call to inner

2. **`execute_inner()`** (lines 286-328): ~43 lines  
   - Contains: Command building, spawn, timeout execution, output parsing, error handling

### Required Fix
Refactor into smaller functions following Single Responsibility Principle:

```
execute()
├── validate_preconditions()     <- NEW: extract validation
├── set_executing_state()       <- NEW: state management  
├── execute_inner()
│   ├── spawn_command()         <- NEW: extract spawn logic
│   ├── wait_with_timeout()     <- NEW: extract timeout handling
│   └── parse_output()         <- NEW: extract parsing
└── reset_executing_state()     <- NEW: state management
```

### Contract Impact
- Preconditions P1, P2, P3 still enforced
- Postconditions Q1, Q2, Q3 still enforced
- Invariants I1, I2, I3 still maintained

---

## Defect 2: Magic Number -1 Without Documentation

**Severity**: MEDIUM  
**Phase**: 4 - DDD Smells

### Location
Line 308: `exit_code: output.status.code().unwrap_or(-1)`

### Description
When `ProcessOutput::status.code()` returns `None` (which happens on abnormal termination like signal kills), the code falls back to `-1`. This magic number is:
- Undocumented in code
- Undocumented in contract
- Ambiguous: does -1 mean "unknown", "signal", or "error"?

### Contract Gap
The contract (Q2: "Exit code reflected in output") does NOT specify behavior when exit code is unavailable. The implementation chose `-1` arbitrarily.

### Required Fix
**Option A** (Preferred): Document the behavior
```rust
/// Exit code, or -1 if unavailable (e.g., process killed by signal)
exit_code: output.status.code().unwrap_or(-1)
```

**Option B** (More Correct): Use a newtype
```rust
/// Represents an exit code, including the "unavailable" state
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExitCode(i32);

impl ExitCode {
    pub const UNAVAILABLE: Self = Self(-1);
    pub fn code(self) -> i32 { self.0 }
}
```

---

## Test Coverage Status
All 15 contract tests are present and passing. No test gaps found.

---

## Recommendations

### Refactoring Priority
1. **Immediate**: Fix function length violations (Defect 1)
2. **Optional**: Add documentation for -1 (Defect 2)

### After Fix
- Re-run: `cargo clippy -p nu_runner -- -D warnings`
- Verify: All tests still pass
- Check: Function counts now under 25 lines each
