# Test Defects - Bead seshat-2wb

## Summary

**CRITICAL FINDING**: The martin-fowler-tests.md is a TEST PLAN WITHOUT ANY IMPLEMENTED RUST TESTS. While the contract.md is well-structured and the UpdateNodeStyle implementation exists, there are no executable Rust tests to verify the behavior. This violates the Testing Trophy requirement for "Real Execution".

---

## Critical Defects

### DEFECT-001: No Executable Tests Exist for UpdateNodeStyle
**Severity**: CRITICAL  
**Category**: Missing Implementation  
**Doctrine Violated**: Testing Trophy, Dan North BDD, Dave Farley ATDD, Kent Beck TDD

The martin-fowler-tests.md (lines 3-108) describes numerous tests:
- Happy path tests (lines 3-8): `test_update_node_style_variant_constructable_with_valid_fields`, etc.
- Error path tests (lines 10-13)
- Edge case tests (lines 15-18)
- Given-When-Then scenarios (lines 51-89)

**NONE of these tests exist in the codebase.**

| Test Described in Plan | Status |
|-----------------------|--------|
| `test_update_node_style_variant_constructable_with_valid_fields` | ❌ Does NOT exist |
| `test_update_node_style_serializes_to_correct_json` | ❌ Does NOT exist |
| `test_update_node_style_deserializes_from_valid_json` | ❌ Does NOT exist |
| `test_update_node_style_kind_returns_node` | ❌ Does NOT exist |
| `test_update_node_style_all_four_style_variants` | ❌ Does NOT exist |
| Scenario tests (lines 53-89) | ❌ Do NOT exist |

**Evidence**:
```bash
# No UpdateNodeStyle tests found in regular tests:
$ grep -r "test_update_node_style" diagram_tool/src/
# Returns: No matches

# Only Kani proofs exist, not real tests:
$ grep -n "given_all_domain_op_variants" diagram_tool/src/models/envelope.rs
1567: fn given_all_domain_op_variants_exhaustive_match_then_all_cases_handled() {
```

The exhaustive match test (line 1567) does include UpdateNodeStyle BUT:
1. Has `#[cfg(kani)]` - only runs with Kani model checker, NOT `cargo test`
2. Is a formal verification proof, not an executable integration/unit test
3. Tests exhaustiveness of match statement, NOT serialization/deserialization behavior

---

### DEFECT-002: No Serialization/Deserialization Tests
**Severity**: CRITICAL  
**Category**: Missing Coverage  
**Doctrine Violated**: Testing Trophy, Contract Postconditions Q2/Q3

The contract specifies:
- Q2: Serialization works - serializes to JSON with "op_type": "update_node_style"
- Q3: Deserialization works - JSON with "op_type" deserializes to DomainOp::UpdateNodeStyle
- I2: Serialization roundtrip - serialize then deserialize yields equivalent DomainOp

**NONE of these are verified by executable tests:**
- No test verifies JSON output format
- No test verifies deserialization from valid JSON
- No test verifies roundtrip integrity

---

### DEFECT-003: No Projection/Apply Operation Tests
**Severity**: HIGH  
**Category**: Missing Coverage  
**Doctrine Violated**: Testing Trophy, Contract Invariant I1

The contract specifies:
- I1: DomainOp completeness - All DomainOp variants are handled in apply_operation dispatch

The projection tests (diagram_tool/src/models/projection/tests.rs) contain NO regular tests:
- All tests have `#[cfg(kani)]` attribute
- No test verifies `apply_operation(DomainOp::UpdateNodeStyle)` works end-to-end

---

### DEFECT-004: No DSL Layer for ATDD
**Severity**: HIGH  
**Category**: ATDD Violation  
**Doctrine Violated**: Dave Farley ATDD

Per Dave Farley: Tests must separate WHAT (intent) from HOW (implementation) via DSL.

The martin-fowler-tests.md does NOT describe a DSL layer. Tests directly call internal functions like `apply_operation`, `DomainOp::UpdateNodeStyle::new()`, etc.

This violates ATDD because:
- Tests are tightly coupled to implementation details
- Tests would break if refactoring occurs
- No business-readable abstraction exists

---

### DEFECT-005: Compilation Error Blocks Test Execution
**Severity**: HIGH  
**Category**: Infrastructure  
**Doctrine Violated**: Testing Trophy

The codebase has duplicate test definitions in commands.rs:
```
error[E0428]: the name `test_apply_copy_selection_returns_false_when_no_selection` is defined multiple times
```

This prevents `cargo test` from running at all, blocking verification of any tests.

---

## Doctrinal Violations Summary

| Doctrine | Status | Evidence |
|----------|--------|----------|
| Dan North BDD | ❌ FAIL | No executable tests - only a test plan |
| Dave Farley ATDD | ❌ FAIL | No DSL layer, tests leak implementation |
| Kent Beck TDD | tests to evaluate isolation/determinism |
 ❌ FAIL | No| Testing Trophy | ❌ FAIL | No real execution possible - only Kani proofs exist |
| Combinatorial Coverage | ❌ N/A | Cannot evaluate without tests |

---

## Required Actions

1. **Fix compilation error** in commands.rs (duplicate test definitions)

2. **Implement regular Rust tests** (not Kani proofs) for UpdateNodeStyle:
   - `test_update_node_style_serialization_box/cloud/cylinder/dashed` - verify JSON output format
   - `test_update_node_style_deserialization_*` - verify deserialization works
   - `test_update_node_style_roundtrip` - verify I2 serialization roundtrip
   - `test_update_node_style_projection` - verify I1 apply_operation works

3. **Add DSL layer** per Dave Farley ATDD:
   - Create test helper functions that abstract implementation details
   - Example: `fn node_with_style(id: &str, style: NodeStyle) -> DiagramDocument`

4. **Run tests with cargo test** to verify they execute (not just Kani)

---

## Verdict

**STATUS: REJECTED**

The martin-fowler-tests.md describes a test plan but the tests do NOT exist as executable Rust code. Per Testing Trophy: "Run the REAL thing first" - there is nothing to run. Per Dave Farley: Tests must separate WHAT from HOW - the DSL doesn't exist. Per Dan North: "Executable Specifications" - these are not executable. Per Kent Beck: Tests must be isolated and deterministic - none exist.

---

*Generated: 2026-03-12*  
*Reviewer: test-reviewer skill*
