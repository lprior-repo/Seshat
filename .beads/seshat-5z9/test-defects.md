# Test Defects Report: seshat-5z9

**Reviewer:** test-reviewer skill  
**Date:** 2026-03-12  
**Bead:** seshat-5z9  
**Feature:** Copy and Paste (CLP-001 to CLP-005)

---

## Critical Defects

### DEFECT-001: Tests Not Runnable with `cargo test`

**Severity:** CRITICAL  
**Location:** `diagram_tool/src/models/clipboard_contract_tests.rs`

**Issue:** All test functions are wrapped in `#[cfg(kani)]` attribute, making them Kani model-checking proofs only. They cannot be executed with standard `cargo test`.

**Evidence:**
- Line 51: `#[cfg(kani)]` on `test_clp001_copy_paste_single_node_creates_new_node_with_new_id`
- Line 84: `#[cfg(kani)]` on `test_clp002_copy_paste_multiple_nodes_preserves_edges_and_remaps_ids`
- Line 119: `#[cfg(kani)]` on `test_clp003_copy_paste_subgraph_preserves_parent_child_relationships`
- All 14 tests in this file have `#[cfg(kani)]`

**Violation:** Testing Trophy philosophy demands **Real Execution** - tests must run with `cargo test` to validate the system actually works.

**Required Fix:** Remove `#[cfg(kani)]` and `#[kani::proof]` attributes from all test functions to make them standard Rust tests.

---

### DEFECT-002: Missing Copy/Paste Round Trip Integration Test

**Severity:** CRITICAL  
**Location:** Test plan not implemented

**Issue:** The user requirement states "Integration test for copy/paste round trip" but no such test exists that:
1. Copies nodes to clipboard
2. Pastes them
3. Copies the new pasted nodes again
4. Pastes again to verify round-trip behavior

**Required Fix:** Add a test that performs a complete copy→paste→copy→paste cycle and verifies:
- Original nodes remain unchanged after first copy
- First paste creates new nodes with new IDs
- Second copy captures the pasted nodes
- Second paste creates another set of new nodes with incremented offsets

---

### DEFECT-003: Empty Contract Verification Tests

**Severity:** HIGH  
**Location:** `clipboard_contract_tests.rs` lines 275-281

**Issue:** Test `test_q1_violation_returns_postcondition_error_for_changed_original_id` is empty - it has no assertions.

**Evidence:**
```rust
#[cfg(kani)]
#[kani::proof]
#[test]
fn test_q1_violation_returns_postcondition_error_for_changed_original_id() {
    // Since we test our actual implementation, the best way to verify this contract
    // is to see that our copy function DOES NOT violate it.
    // A direct mock violation test isn't strictly necessary if the implementation handles it.
    // But per contract, we expect pure behavior.
}
```

**Required Fix:** Implement a proper test that verifies Q1 (original nodes remain unchanged after copy).

---

### DEFECT-004: Missing Contract Violation Tests

**Severity:** HIGH  
**Location:** Test plan vs implementation gap

**Issue:** The martin-fowler-tests.md specifies these contract violation tests that are NOT implemented:

| Test ID | Description | Status |
|---------|-------------|--------|
| test_q2_violation_returns_postcondition_error_for_missing_edges | Verifies edges between selected nodes are copied | NOT IMPLEMENTED |
| test_q3_violation_returns_postcondition_error_for_nodes_not_deleted | Verifies cut removes original nodes | NOT IMPLEMENTED |
| test_q4_violation_returns_duplicate_id_error | Verifies paste generates unique IDs | NOT IMPLEMENTED |
| test_q5_violation_returns_postcondition_error_for_zero_offset | Verifies paste offset increments | NOT IMPLEMENTED |

**Required Fix:** Implement all missing contract violation tests.

---

## Moderate Defects

### DEFECT-005: No Given-When-Then Structure in Tests

**Severity:** MODERATE  
**Location:** All test files

**Issue:** Per Dan North's BDD doctrine, tests should have expressive Given-When-Then structure. Current test names like `test_clp001_copy_paste_single_node_creates_new_node_with_new_id` are descriptive but don't follow the GWT pattern.

**Example of Expected Format:**
```rust
#[test]
fn given_document_with_node_selected_when_copy_and_paste_then_creates_new_node_with_new_id() {
    // Given: A document with Node A selected
    // When: The copy operation is performed, followed by paste
    // Then: A new node is created with a distinct ID
}
```

**Required Fix:** Restructure test names and add comments to follow GWT pattern.

---

### DEFECT-006: No DSL Layer for ATDD

**Severity:** MODERATE  
**Location:** Test implementation

**Issue:** Per Dave Farley's ATDD doctrine, tests should separate WHAT (intent) from HOW (implementation). Current tests directly call implementation functions (`copy`, `cut`, `paste`) without a DSL layer.

**Required Fix:** Consider creating a thin DSL layer (e.g., `clipboard_dsl.rs`) that provides behavior-focused functions like:
- `copy_to_clipboard(doc, selection)` → "Copy selection to clipboard"
- `paste_from_clipboard(doc)` → "Paste from clipboard"
- `verify_new_ids_exist(doc, original_ids)` → "Verify new IDs are created"

---

## Summary

| Priority | Count | Status |
|----------|-------|--------|
| Critical | 2 | MUST FIX |
| High | 2 | SHOULD FIX |
| Moderate | 2 | NICE TO HAVE |

**Overall Assessment:** The test plan is FLAWED. The tests are not executable with `cargo test` due to Kani-only configuration, violating the Testing Trophy "Real Execution" principle. Additionally, critical tests are missing or empty.

---

**Recommendation:** REJECT the test plan until:
1. All `#[cfg(kani)]` attributes are removed to enable `cargo test` execution
2. Copy/paste round trip integration test is added
3. All contract violation tests are implemented
